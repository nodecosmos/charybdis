use crate::callbacks::Callbacks;
use crate::errors::CharybdisError;
use crate::iterator::CharybdisModelIterator;
use crate::model::{BaseModel, Model};
use crate::operations::OperationsWithCallbacks;
use crate::stream::CharybdisModelStream;
use crate::{Consistency, SerializeRow};
use scylla::_macro_internal::{RowSerializationContext, RowWriter, SerializationError};
use scylla::execution_profile::ExecutionProfileHandle;
use scylla::history::HistoryListener;
use scylla::query::Query;
use scylla::retry_policy::RetryPolicy;
use scylla::statement::SerialConsistency;
use scylla::{Bytes, CachingSession, IntoTypedRows, QueryResult};
use std::sync::Arc;
use std::time::Duration;

pub struct SingleRow<M: BaseModel>(M);
pub struct RowStream<M: BaseModel>(CharybdisModelStream<M>);
pub struct Paged<M: BaseModel>(CharybdisModelIterator<M>, Option<Bytes>);
pub struct QueryResultWrapper(QueryResult);

pub trait ResponseType {
    type Output;
}

impl<M: BaseModel> ResponseType for SingleRow<M> {
    type Output = M;
}

impl<M: BaseModel> ResponseType for RowStream<M> {
    type Output = CharybdisModelStream<M>;
}

impl<M: BaseModel> ResponseType for Paged<M> {
    type Output = (CharybdisModelIterator<M>, Option<Bytes>);
}

impl ResponseType for QueryResultWrapper {
    type Output = QueryResult;
}

pub trait QueryExecutor<M: BaseModel>: ResponseType {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor<M>>(
        query: CharybdisQuery<'_, Val, M, RtQe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>;
}

impl<M: BaseModel> QueryExecutor<M> for SingleRow<M> {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor<M>>(
        query: CharybdisQuery<'_, Val, M, RtQe>,
        session: &CachingSession,
    ) -> Result<M, CharybdisError> {
        let row = session.execute(query.inner, query.values).await?;
        let res = row.first_row_typed::<M>()?;

        Ok(res)
    }
}

impl<M: BaseModel> QueryExecutor<M> for RowStream<M> {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor<M>>(
        query: CharybdisQuery<'_, Val, M, RtQe>,
        session: &CachingSession,
    ) -> Result<CharybdisModelStream<M>, CharybdisError> {
        let rows = session.execute_iter(query.inner, query.values).await?.into_typed::<M>();

        Ok(CharybdisModelStream::from(rows))
    }
}

impl<M: BaseModel> QueryExecutor<M> for Paged<M> {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor<M>>(
        query: CharybdisQuery<'_, Val, M, RtQe>,
        session: &CachingSession,
    ) -> Result<(CharybdisModelIterator<M>, Option<Bytes>), CharybdisError> {
        let res = session
            .execute_paged(query.inner, query.values, query.paging_state)
            .await?;
        let paging_state = res.paging_state.clone();
        let rows = res.rows()?;
        let typed_rows = CharybdisModelIterator::from(rows.into_typed());

        Ok((typed_rows, paging_state))
    }
}

impl<M: BaseModel> QueryExecutor<M> for QueryResultWrapper {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor<M>>(
        query: CharybdisQuery<'_, Val, M, RtQe>,
        session: &CachingSession,
    ) -> Result<QueryResult, CharybdisError> {
        session
            .execute(query.inner, query.values)
            .await
            .map_err(CharybdisError::from)
    }
}

#[derive(Default)]
pub(crate) enum QueryValue<'a, Val: SerializeRow, M: BaseModel> {
    Owned(Val),
    Ref(&'a Val),
    PrimaryKey(M::PrimaryKey),
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
            QueryValue::Model(val) => val.serialize(ctx, writer),
            QueryValue::Empty => Ok(()),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            QueryValue::Owned(val) => val.is_empty(),
            QueryValue::Ref(val) => val.is_empty(),
            QueryValue::PrimaryKey(val) => val.is_empty(),
            QueryValue::Model(val) => val.is_empty(),
            QueryValue::Empty => true,
        }
    }
}

pub struct CharybdisQuery<'a, Val: SerializeRow, M: BaseModel, RtQe: ResponseType + QueryExecutor<M>> {
    inner: Query,
    paging_state: Option<Bytes>,
    values: QueryValue<'a, Val, M>,
    _phantom: std::marker::PhantomData<RtQe>,
}

impl<'a, Val: SerializeRow, M: BaseModel, RtQe: ResponseType + QueryExecutor<M>> CharybdisQuery<'a, Val, M, RtQe> {
    pub(crate) fn new(query: impl Into<String>, values: QueryValue<'a, Val, M>) -> Self {
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

    pub async fn execute(self, session: &CachingSession) -> Result<RtQe::Output, CharybdisError> {
        RtQe::execute(self, session).await
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

pub struct CharybdisCbQuery<'a, M: Model + Callbacks, Val: SerializeRow> {
    inner: CharybdisQuery<'a, Val, M, QueryResultWrapper>,
    operation: OperationsWithCallbacks,
    extension: &'a M::Extension,
    model: &'a mut M,
}

impl<'a, M: Model + Callbacks, Val: SerializeRow> CharybdisCbQuery<'a, M, Val> {
    pub fn new(
        query: impl Into<String>,
        operation: OperationsWithCallbacks,
        extension: &'a M::Extension,
        model: &'a mut M,
    ) -> Self {
        Self {
            inner: CharybdisQuery::new(query, QueryValue::default()),
            operation,
            extension,
            model,
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

    pub async fn execute(self, session: &CachingSession) -> Result<QueryResult, M::Error> {
        match self.operation {
            OperationsWithCallbacks::Insert => self.model.before_insert(session, self.extension).await?,
            OperationsWithCallbacks::Update => self.model.before_update(session, self.extension).await?,
            OperationsWithCallbacks::Delete => self.model.before_delete(session, self.extension).await?,
        }

        let value = match self.operation {
            OperationsWithCallbacks::Insert | OperationsWithCallbacks::Update => QueryValue::Model(self.model),
            OperationsWithCallbacks::Delete => QueryValue::PrimaryKey(self.model.primary_key_values()),
        };

        let res = self.inner.values(value).execute(session).await?;

        match self.operation {
            OperationsWithCallbacks::Insert => self.model.after_insert(session, self.extension).await?,
            OperationsWithCallbacks::Update => self.model.after_update(session, self.extension).await?,
            OperationsWithCallbacks::Delete => self.model.after_delete(session, self.extension).await?,
        }

        Ok(res)
    }
}
