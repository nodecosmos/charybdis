use crate::errors::CharybdisError;
use crate::iterator::CharybdisModelIterator;
use crate::model::BaseModel;
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

pub trait QueryExecutor: ResponseType {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor>(
        query: CharybdisQuery<'_, Val, RtQe>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>;
}

impl<M: BaseModel> QueryExecutor for SingleRow<M> {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor>(
        query: CharybdisQuery<'_, Val, RtQe>,
        session: &CachingSession,
    ) -> Result<M, CharybdisError> {
        let row = session.execute(query.inner, query.values).await?;
        let res = row.first_row_typed::<M>()?;

        Ok(res)
    }
}

impl<M: BaseModel> QueryExecutor for RowStream<M> {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor>(
        query: CharybdisQuery<'_, Val, RtQe>,
        session: &CachingSession,
    ) -> Result<CharybdisModelStream<M>, CharybdisError> {
        let rows = session.execute_iter(query.inner, query.values).await?.into_typed::<M>();

        Ok(CharybdisModelStream::from(rows))
    }
}

impl<M: BaseModel> QueryExecutor for Paged<M> {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor>(
        query: CharybdisQuery<'_, Val, RtQe>,
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

impl QueryExecutor for QueryResultWrapper {
    async fn execute<Val: SerializeRow, RtQe: ResponseType + QueryExecutor>(
        query: CharybdisQuery<'_, Val, RtQe>,
        session: &CachingSession,
    ) -> Result<QueryResult, CharybdisError> {
        session
            .execute(query.inner, query.values)
            .await
            .map_err(CharybdisError::from)
    }
}

pub(crate) enum QueryValue<'a, Val: SerializeRow> {
    Owned(Val),
    Ref(&'a Val),
}

impl<Val: SerializeRow> SerializeRow for QueryValue<'_, Val> {
    fn serialize(&self, ctx: &RowSerializationContext<'_>, writer: &mut RowWriter) -> Result<(), SerializationError> {
        match self {
            QueryValue::Owned(val) => val.serialize(ctx, writer),
            QueryValue::Ref(val) => val.serialize(ctx, writer),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            QueryValue::Owned(val) => val.is_empty(),
            QueryValue::Ref(val) => val.is_empty(),
        }
    }
}

pub struct CharybdisQuery<'a, Val: SerializeRow, RtQe: ResponseType + QueryExecutor> {
    inner: Query,
    paging_state: Option<Bytes>,
    values: QueryValue<'a, Val>,
    _phantom: std::marker::PhantomData<RtQe>,
}

impl<'a, Val: SerializeRow, RtQe: ResponseType + QueryExecutor> CharybdisQuery<'a, Val, RtQe> {
    pub(crate) fn new(query: impl Into<String>, values: QueryValue<'a, Val>) -> Self {
        Self {
            inner: Query::new(query),
            paging_state: None,
            values,
            _phantom: Default::default(),
        }
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
