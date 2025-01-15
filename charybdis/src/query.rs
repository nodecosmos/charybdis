use crate::callbacks::{CallbackAction, Callbacks};
use crate::errors::CharybdisError;
use crate::iterator::CharybdisModelIterator;
use crate::model::BaseModel;
use crate::options::{Consistency, ExecutionProfileHandle, HistoryListener, RetryPolicy, SerialConsistency};
use crate::stream::CharybdisModelStream;
use scylla::query::Query;
use scylla::serialize::row::{RowSerializationContext, SerializeRow};
use scylla::serialize::{writers::RowWriter, SerializationError};
use scylla::statement::{PagingState, PagingStateResponse};
use scylla::transport::query_result::FirstRowError;
use scylla::{CachingSession, QueryResult};
use std::sync::Arc;
use std::time::Duration;

pub struct ModelRow;
pub struct OptionalModelRow;
pub struct ModelStream;
pub struct ModelPaged;
pub struct ModelMutation;

pub trait QueryType<M: BaseModel> {
    type Output;
}

impl<M: BaseModel> QueryType<M> for ModelRow {
    type Output = M;
}

impl<M: BaseModel> QueryType<M> for OptionalModelRow {
    type Output = Option<M>;
}

impl<M: 'static + BaseModel> QueryType<M> for ModelStream {
    type Output = CharybdisModelStream<M>;
}

impl<M: BaseModel> QueryType<M> for ModelPaged {
    type Output = (CharybdisModelIterator<M>, PagingStateResponse);
}

impl<M: BaseModel> QueryType<M> for ModelMutation {
    type Output = QueryResult;
}

pub trait QueryExecutor<M: BaseModel>: QueryType<M> {
    async fn execute<Val, Qe>(
        query: CharybdisQuery<Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor<M>;
}

impl<M: BaseModel> QueryExecutor<M> for ModelRow {
    async fn execute<Val, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor<M>,
    {
        let res = session
            .execute_unpaged(query.inner, query.values)
            .await
            .map_err(|e| CharybdisError::QueryError(query.query_string, e))?;

        let res = res
            .into_rows_result()
            .map_err(|e| CharybdisError::IntoRowsResultError(query.query_string, e))?;

        let row = res.first_row::<M>().map_err(|e| match e {
            FirstRowError::RowsEmpty => CharybdisError::NotFoundError(query.query_string),
            _ => CharybdisError::FirstRowError(query.query_string, e),
        })?;

        Ok(row)
    }
}

impl<M: BaseModel> QueryExecutor<M> for OptionalModelRow {
    async fn execute<Val, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor<M>,
    {
        let res = session
            .execute_unpaged(query.inner, query.values)
            .await
            .map_err(|e| CharybdisError::QueryError(query.query_string, e))?
            .into_rows_result()
            .map_err(|e| CharybdisError::IntoRowsResultError(query.query_string, e))?
            .maybe_first_row::<M>()
            .map_err(|e| CharybdisError::MaybeFirstRowError(query.query_string, e))?;

        Ok(res)
    }
}

impl<M: BaseModel + 'static> QueryExecutor<M> for ModelStream {
    async fn execute<Val, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor<M>,
    {
        let res = session
            .execute_iter(query.inner, query.values)
            .await
            .map_err(|e| CharybdisError::QueryError(query.query_string, e))?
            .rows_stream::<M>()
            .map_err(|e| CharybdisError::TypeCheckError(query.query_string, e))?;

        let mut stream = CharybdisModelStream::from(res);
        stream.query_string(query.query_string);

        Ok(stream)
    }
}

impl<M: BaseModel> QueryExecutor<M> for ModelPaged {
    async fn execute<Val, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor<M>,
    {
        let res = session
            .execute_single_page(query.inner, query.values, query.paging_state)
            .await
            .map_err(|e| CharybdisError::QueryError(query.query_string, e))?;
        let psr = res.1;

        let rows_res = res
            .0
            .into_rows_result()
            .map_err(|e| CharybdisError::IntoRowsResultError(query.query_string, e))?;

        let rows = rows_res
            .rows()
            .map_err(|e| CharybdisError::RowsError(query.query_string, e))?;

        let models_result: Result<Vec<M>, CharybdisError> = rows
            .map(|row_result| row_result.map_err(|e| CharybdisError::DeserializationError(query.query_string, e)))
            .collect();

        let models = models_result?;

        let mut iterator = CharybdisModelIterator::from(models);
        iterator.query_string(query.query_string);

        Ok((iterator, psr))
    }
}

impl<M: BaseModel> QueryExecutor<M> for ModelMutation {
    async fn execute<Val, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor<M>,
    {
        let res = session
            .execute_unpaged(query.inner, query.values)
            .await
            .map_err(|e| CharybdisError::QueryError(query.query_string, e))?;

        Ok(res)
    }
}

#[derive(Default)]
pub enum QueryValue<'a, Val: SerializeRow, M: BaseModel> {
    Owned(Val),
    Ref(&'a Val),
    PrimaryKey(M::PrimaryKey),
    PartitionKey(M::PartitionKey),
    Model(&'a M),
    ModelOwned(M),
    #[default]
    Empty,
}

impl<Val: SerializeRow, M: BaseModel> SerializeRow for QueryValue<'_, Val, M> {
    fn serialize(&self, ctx: &RowSerializationContext<'_>, writer: &mut RowWriter) -> Result<(), SerializationError> {
        match self {
            QueryValue::Owned(val) => val.serialize(ctx, writer),
            QueryValue::Ref(val) => val.serialize(ctx, writer),
            QueryValue::PrimaryKey(val) => val.serialize(ctx, writer),
            QueryValue::PartitionKey(val) => val.serialize(ctx, writer),
            QueryValue::Model(val) => val.serialize(ctx, writer),
            QueryValue::ModelOwned(val) => val.serialize(ctx, writer),
            QueryValue::Empty => Ok(()),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            QueryValue::Owned(val) => val.is_empty(),
            QueryValue::Ref(val) => val.is_empty(),
            QueryValue::PrimaryKey(val) => val.is_empty(),
            QueryValue::PartitionKey(val) => val.is_empty(),
            QueryValue::Model(val) => val.is_empty(),
            QueryValue::ModelOwned(val) => val.is_empty(),
            QueryValue::Empty => true,
        }
    }
}

pub struct CharybdisQuery<'a, Val: SerializeRow, M: BaseModel, Qe: QueryExecutor<M>> {
    inner: Query,
    paging_state: PagingState,
    pub(crate) query_string: &'static str,
    pub(crate) values: QueryValue<'a, Val, M>,
    _phantom: std::marker::PhantomData<Qe>,
}

impl<'a, Val: SerializeRow, M: BaseModel, Qe: QueryExecutor<M>> CharybdisQuery<'a, Val, M, Qe> {
    pub fn new(query: &'static str, values: QueryValue<'a, Val, M>) -> Self {
        Self {
            inner: Query::new(query),
            query_string: query,
            values,
            paging_state: PagingState::start(),
            _phantom: Default::default(),
        }
    }

    pub(crate) fn values(mut self, values: QueryValue<'a, Val, M>) -> Self {
        self.values = values;

        self
    }

    pub fn page_size(mut self, page_size: i32) -> Self {
        self.inner.set_page_size(page_size);
        self
    }

    pub fn consistency(mut self, consistency: Consistency) -> Self {
        self.inner.set_consistency(consistency);
        self
    }

    pub fn serial_consistency(mut self, consistency: Option<SerialConsistency>) -> Self {
        self.inner.set_serial_consistency(consistency);
        self
    }

    pub fn paging_state(mut self, paging_state: PagingState) -> Self {
        self.paging_state = paging_state;
        self
    }

    pub fn idempotent(mut self, is_idempotent: bool) -> Self {
        self.inner.set_is_idempotent(is_idempotent);
        self
    }

    pub fn trace(mut self, is_tracing: bool) -> Self {
        self.inner.set_tracing(is_tracing);
        self
    }

    pub fn timestamp(mut self, timestamp: Option<i64>) -> Self {
        self.inner.set_timestamp(timestamp);
        self
    }

    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.inner.set_request_timeout(timeout);
        self
    }

    pub fn retry_policy(mut self, retry_policy: Option<Arc<dyn RetryPolicy>>) -> Self {
        self.inner.set_retry_policy(retry_policy);
        self
    }

    pub fn history_listener(mut self, history_listener: Arc<dyn HistoryListener>) -> Self {
        self.inner.set_history_listener(history_listener);
        self
    }

    pub fn remove_history_listener(mut self) -> Self {
        self.inner.remove_history_listener();
        self
    }

    pub fn profile_handle(mut self, profile_handle: Option<ExecutionProfileHandle>) -> Self {
        self.inner.set_execution_profile_handle(profile_handle);
        self
    }

    pub async fn execute(self, session: &CachingSession) -> Result<Qe::Output, CharybdisError> {
        Qe::execute(self, session).await
    }
}

macro_rules! delegate_inner_query_methods {
    ($($method:ident($($param_name:ident: $param_type:ty),*)  ),* $(,)? ) => {
        $(
            pub fn $method(mut self, $($param_name: $param_type),*) -> Self {
                self.inner = self.inner.$method($($param_name),*);
                self
            }
        )*
    };
}

pub struct CharybdisCbQuery<'a, M: Callbacks, CbA: CallbackAction<M>, Val: SerializeRow> {
    inner: CharybdisQuery<'a, Val, M, ModelMutation>,
    model: &'a mut M,
    extension: &'a M::Extension,
    _phantom: std::marker::PhantomData<CbA>,
}

impl<'a, M: Callbacks, CbA: CallbackAction<M>, Val: SerializeRow> CharybdisCbQuery<'a, M, CbA, Val> {
    pub(crate) fn new(query: &'static str, model: &'a mut M, extension: &'a M::Extension) -> Self {
        Self {
            inner: CharybdisQuery::new(query, QueryValue::default()),
            model,
            extension,
            _phantom: Default::default(),
        }
    }

    delegate_inner_query_methods! {
        page_size(page_size: i32),
        consistency(consistency: Consistency),
        serial_consistency(consistency: Option<SerialConsistency>),
        paging_state(paging_state: PagingState),
        idempotent(is_idempotent: bool),
        trace(is_tracing: bool),
        timestamp(timestamp: Option<i64>),
        timeout(timeout: Option<Duration>),
        retry_policy(retry_policy: Option<Arc<dyn RetryPolicy>>),
        history_listener(history_listener: Arc<dyn HistoryListener>),
        remove_history_listener(),
        profile_handle(profile_handle: Option<ExecutionProfileHandle>)
    }

    pub async fn execute(self, session: &CachingSession) -> Result<QueryResult, M::Error> {
        CbA::before_execute(self.model, session, self.extension).await?;

        let query_value = CbA::query_value(self.model);
        let res = self.inner.values(query_value).execute(session).await?;

        CbA::after_execute(self.model, session, self.extension).await?;

        Ok(res)
    }
}
