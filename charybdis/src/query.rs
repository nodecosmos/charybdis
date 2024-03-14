use crate::callbacks::{CallbackAction, CallbackContext, Callbacks};
use crate::errors::CharybdisError;
use crate::iterator::CharybdisModelIterator;
use crate::model::BaseModel;
use crate::options::{Consistency, ExecutionProfileHandle, HistoryListener, RetryPolicy, SerialConsistency};
use crate::saga::Saga;
use crate::stream::CharybdisModelStream;
use crate::SerializeRow;
use scylla::_macro_internal::{RowSerializationContext, RowWriter, SerializationError};
use scylla::query::Query;
use scylla::{Bytes, CachingSession, IntoTypedRows, QueryResult};
use std::sync::Arc;
use std::time::Duration;

pub struct ModelRow<M: BaseModel>(M);
pub struct OptionalModelRow<M: BaseModel>(Option<M>);
pub struct ModelStream<M: BaseModel>(CharybdisModelStream<M>);
pub struct ModelPaged<M: BaseModel>(CharybdisModelIterator<M>, Option<Bytes>);
pub struct ModelMutation(QueryResult);

pub trait QueryType {
    type Output;
}

impl<M: BaseModel> QueryType for ModelRow<M> {
    type Output = M;
}

impl<M: BaseModel> QueryType for OptionalModelRow<M> {
    type Output = Option<M>;
}

impl<M: BaseModel> QueryType for ModelStream<M> {
    type Output = CharybdisModelStream<M>;
}

impl<M: BaseModel> QueryType for ModelPaged<M> {
    type Output = (CharybdisModelIterator<M>, Option<Bytes>);
}

impl QueryType for ModelMutation {
    type Output = QueryResult;
}

pub trait QueryExecutor: QueryType {
    async fn execute<Val, M, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor;
}

impl<Bm: BaseModel> QueryExecutor for ModelRow<Bm> {
    async fn execute<Val, M, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor,
    {
        let row = session.execute(query.inner, query.values).await?;
        let res = row.first_row_typed::<Bm>()?;

        Ok(res)
    }
}

impl<Bm: BaseModel> QueryExecutor for OptionalModelRow<Bm> {
    async fn execute<Val, M, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor,
    {
        let row = session.execute(query.inner, query.values).await?;
        let res = row.maybe_first_row_typed::<Bm>()?;

        Ok(res)
    }
}

impl<Bm: BaseModel> QueryExecutor for ModelStream<Bm> {
    async fn execute<Val, M, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor,
    {
        let rows = session
            .execute_iter(query.inner, query.values)
            .await?
            .into_typed::<Bm>();

        Ok(CharybdisModelStream::from(rows))
    }
}

impl<Bm: BaseModel> QueryExecutor for ModelPaged<Bm> {
    async fn execute<Val, M, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor,
    {
        let res = session
            .execute_paged(query.inner, query.values, query.paging_state)
            .await?;
        let paging_state = res.paging_state.clone();
        let rows = res.rows()?;
        let typed_rows = CharybdisModelIterator::from(rows.into_typed());

        Ok((typed_rows, paging_state))
    }
}

impl QueryExecutor for ModelMutation {
    async fn execute<Val, M, Qe>(
        query: CharybdisQuery<'_, Val, M, Qe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>
    where
        M: BaseModel,
        Val: SerializeRow,
        Qe: QueryExecutor,
    {
        session
            .execute(query.inner, query.values)
            .await
            .map_err(CharybdisError::from)
    }
}

#[derive(Default)]
pub enum QueryValue<'a, Val: SerializeRow, M: BaseModel> {
    Owned(Val),
    Ref(&'a Val),
    PrimaryKey(M::PrimaryKey),
    PartitionKey(M::PartitionKey),
    Model(&'a M),
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
            QueryValue::Empty => true,
        }
    }
}

pub struct CharybdisQuery<'a, Val: SerializeRow, M: BaseModel, Qe: QueryExecutor> {
    inner: Query,
    paging_state: Option<Bytes>,
    pub(crate) values: QueryValue<'a, Val, M>,
    _phantom: std::marker::PhantomData<Qe>,
}

impl<'a, Val: SerializeRow, M: BaseModel, Qe: QueryExecutor> CharybdisQuery<'a, Val, M, Qe> {
    pub fn new(query: impl Into<String>, values: QueryValue<'a, Val, M>) -> Self {
        Self {
            inner: Query::new(query),
            paging_state: None,
            values,
            _phantom: Default::default(),
        }
    }

    pub(crate) fn values(mut self, values: QueryValue<'a, Val, M>) -> Self {
        self.values = values;

        self
    }

    pub(crate) fn contents(&self) -> &String {
        &self.inner.contents
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

    pub fn paging_state(mut self, paging_state: Option<Bytes>) -> Self {
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
    pub(crate) fn new(query: impl Into<String>, model: &'a mut M, extension: &'a M::Extension) -> Self {
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
        paging_state(paging_state: Option<Bytes>),
        idempotent(is_idempotent: bool),
        trace(is_tracing: bool),
        timestamp(timestamp: Option<i64>),
        timeout(timeout: Option<Duration>),
        retry_policy(retry_policy: Option<Arc<dyn RetryPolicy>>),
        history_listener(history_listener: Arc<dyn HistoryListener>),
        remove_history_listener(),
        profile_handle(profile_handle: Option<ExecutionProfileHandle>)
    }

    pub async fn execute(mut self, session: &CachingSession) -> Result<QueryResult, M::Error> {
        let mut saga = Saga::new();
        let ctx = CallbackContext::new(session, self.extension, &mut saga);

        // initialize before_<action> saga steps
        CbA::before_execute(&mut self.model, &ctx).await?;

        // execute & drain before_<action> steps
        ctx.saga.execute().await?;

        // execute the query
        let res = self.inner.values(CbA::query_value(self.model)).execute(session).await;

        if let Err(e) = res {
            // recover saga as main query failed
            ctx.saga.execute_compensating_actions().await?;

            return Err(M::Error::from(e));
        }

        // initialize after_<action> saga steps
        CbA::after_execute(&mut self.model, &ctx).await?;

        // execute after_<action> steps
        // if any of the after_<action> steps fail the compensating actions will be executed
        // however, the main query has already been executed and cannot be rolled back, so
        // ATM user has to rollback main query manually
        ctx.saga.execute().await?;

        Ok(res?)
    }
}
