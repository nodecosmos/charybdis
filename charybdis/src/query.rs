use crate::errors::CharybdisError;
use crate::iterator::CharybdisModelIterator;
use crate::model::BaseModel;
use crate::stream::CharybdisModelStream;
use crate::{Consistency, SerializeRow};
use scylla::execution_profile::ExecutionProfileHandle;
use scylla::history::HistoryListener;
use scylla::query::Query;
use scylla::retry_policy::RetryPolicy;
use scylla::statement::SerialConsistency;
use scylla::{Bytes, CachingSession, IntoTypedRows, QueryResult};
use std::sync::Arc;
use std::time::Duration;

pub trait QueryExecutor {
    type Output;

    async fn execute<E: QueryExecutor, Val: SerializeRow>(
        query: CharybdisQuery<E, Val>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError>;
}

/// Single typed row
impl<M: BaseModel> QueryExecutor for M {
    type Output = M;

    async fn execute<E: QueryExecutor, Val: SerializeRow>(
        query: CharybdisQuery<E, Val>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError> {
        let row = session.execute(query.inner, query.values).await?;
        let res = row.first_row_typed::<M>()?;

        Ok(res)
    }
}

/// Multiple typed rows as iterator. Iterator is only used for paging queries, otherwise use stream.
impl<M: BaseModel> QueryExecutor for CharybdisModelIterator<M> {
    type Output = (CharybdisModelIterator<M>, Option<Bytes>);

    async fn execute<E: QueryExecutor, Val: SerializeRow>(
        query: CharybdisQuery<E, Val>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError> {
        let res = session
            .execute_paged(query.inner, query.values, query.paging_state)
            .await?;
        let paging_state = res.paging_state.clone();
        let rows = res.rows()?;
        let typed_rows = CharybdisModelIterator::from(rows.into_typed());

        Ok((typed_rows, paging_state))
    }
}

/// Multiple typed rows as stream.
impl<M: BaseModel> QueryExecutor for CharybdisModelStream<M> {
    type Output = CharybdisModelStream<M>;

    async fn execute<E: QueryExecutor, Val: SerializeRow>(
        query: CharybdisQuery<E, Val>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError> {
        let rows = session.execute_iter(query.inner, query.values).await?.into_typed::<M>();

        Ok(CharybdisModelStream::from(rows))
    }
}

/// Raw query result
impl QueryExecutor for QueryResult {
    type Output = QueryResult;

    async fn execute<E: QueryExecutor, Val: SerializeRow>(
        query: CharybdisQuery<E, Val>,
        session: &CachingSession,
    ) -> Result<Self::Output, CharybdisError> {
        session
            .execute(query.inner, query.values)
            .await
            .map_err(CharybdisError::from)
    }
}

pub struct CharybdisQuery<E: QueryExecutor, V: SerializeRow> {
    inner: Query,
    values: V,
    paging_state: Option<Bytes>,
    phantom: std::marker::PhantomData<E>,
}

impl<E: QueryExecutor, V: SerializeRow> CharybdisQuery<E, V> {
    pub(crate) fn new(query: impl Into<String>, values: V) -> Self {
        Self {
            inner: Query::new(query),
            values,
            paging_state: None,
            phantom: std::marker::PhantomData,
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

    pub fn serial_consistency(mut self, consistency: SerialConsistency) -> Self {
        self.inner.set_serial_consistency(Some(consistency));
        self
    }

    pub fn paging_state(mut self, paging_state: Bytes) -> Self {
        self.paging_state = Some(paging_state);
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

    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.inner.set_timestamp(Some(timestamp));
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner.set_request_timeout(Some(timeout));
        self
    }

    pub fn retry_policy(mut self, retry_policy: Arc<dyn RetryPolicy>) -> Self {
        self.inner.set_retry_policy(Some(retry_policy));
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

    pub fn profile_handle(mut self, profile_handle: ExecutionProfileHandle) -> Self {
        self.inner.set_execution_profile_handle(Some(profile_handle));
        self
    }

    pub async fn execute(self, session: &CachingSession) -> Result<E::Output, CharybdisError> {
        E::execute(self, session).await
    }
}
