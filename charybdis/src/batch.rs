use crate::errors::CharybdisError;
use crate::model::Model;
use crate::options::{Consistency, ExecutionProfileHandle, RetryPolicy, SerialConsistency};
use crate::query::{CharybdisQuery, QueryExecutor, QueryValue};
use scylla::batch::{Batch, BatchType};
use scylla::history::HistoryListener;
use scylla::serialize::row::SerializeRow;
use scylla::{CachingSession, QueryResult};
use std::sync::Arc;

pub struct CharybdisModelBatch<'a, Val: SerializeRow, M: Model> {
    inner: Batch,
    values: Vec<QueryValue<'a, Val, M>>,
}

impl<'a, Val: SerializeRow, M: Model> CharybdisModelBatch<'a, Val, M> {
    pub fn new() -> Self {
        Self {
            inner: Batch::default(),
            values: Vec::new(),
        }
    }

    pub fn unlogged() -> Self {
        Self {
            inner: Batch::new(BatchType::Unlogged),
            values: Vec::new(),
        }
    }

    pub(crate) fn from_batch(batch: &Batch) -> Self {
        Self {
            inner: batch.clone(),
            values: Vec::new(),
        }
    }

    fn append_query_to_batch(&mut self, statement: &str) {
        self.inner.append_statement(statement);
    }

    pub fn consistency(mut self, consistency: Consistency) -> Self {
        self.inner.set_consistency(consistency);
        self
    }

    pub fn serial_consistency(mut self, consistency: Option<SerialConsistency>) -> Self {
        self.inner.set_serial_consistency(consistency);
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

    pub async fn chunked_insert(
        self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M, M> = CharybdisModelBatch::from_batch(&self.inner);

            batch.append_inserts(chunk);

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_insert_if_not_exist(
        self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M, M> = CharybdisModelBatch::from_batch(&self.inner);

            batch.append_inserts_if_not_exist(chunk);

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_update(
        self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M, M> = CharybdisModelBatch::from_batch(&self.inner);

            batch.append_updates(chunk);

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete(
        self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M, M> = CharybdisModelBatch::from_batch(&self.inner);

            for model in chunk {
                batch.append_delete(model);
            }

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete_by_partition_key(
        self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M, M> = CharybdisModelBatch::from_batch(&self.inner);

            batch.append_deletes_by_partition_key(chunk);

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_statements(
        self,
        db_session: &CachingSession,
        statement: &str,
        mut values: Vec<Val>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        while !values.is_empty() {
            let chunk: Vec<Val> = values.drain(..std::cmp::min(chunk_size, values.len())).collect();
            let batch: CharybdisModelBatch<Val, M> = CharybdisModelBatch::from_batch(&self.inner);

            batch.append_statements(statement, chunk)?;

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub fn append_statements(&self, statement: &str, values: Vec<Val>) -> Result<(), CharybdisError> {
        let mut batch: CharybdisModelBatch<Val, M> = CharybdisModelBatch::from_batch(&self.inner);

        for val in values {
            batch.append_statement(statement, val);
        }

        Ok(())
    }

    pub fn append_insert(&mut self, model: &'a M) -> &mut Self {
        self.append_query_to_batch(M::INSERT_QUERY);
        self.values.push(QueryValue::Model(model));
        self
    }

    pub fn append_inserts(&mut self, iter: &'a [M]) -> &mut Self {
        for model in iter {
            self.append_insert(model);
        }
        self
    }

    pub fn append_insert_if_not_exist(&mut self, model: &'a M) -> &mut Self {
        self.append_query_to_batch(M::INSERT_IF_NOT_EXIST_QUERY);
        self.values.push(QueryValue::Model(model));
        self
    }

    pub fn append_inserts_if_not_exist(&mut self, iter: &'a [M]) -> &mut Self {
        for model in iter {
            self.append_insert_if_not_exist(model);
        }
        self
    }

    pub fn append_update(&mut self, model: &'a M) -> &mut Self {
        self.append_query_to_batch(M::UPDATE_QUERY);
        self.values.push(QueryValue::Model(model));
        self
    }

    pub fn append_updates(&mut self, iter: &'a [M]) -> &mut Self {
        for model in iter {
            self.append_update(model);
        }
        self
    }

    pub fn append_delete(&mut self, model: &M) -> &mut Self {
        self.append_query_to_batch(M::DELETE_QUERY);
        self.values.push(QueryValue::PrimaryKey(model.primary_key_values()));
        self
    }

    pub fn append_deletes(&mut self, iter: &[M]) -> &mut Self {
        for model in iter {
            self.append_delete(model);
        }
        self
    }

    pub fn append_delete_by_partition_key(&mut self, model: &'a M) -> &mut Self {
        self.append_query_to_batch(M::DELETE_BY_PARTITION_KEY_QUERY);
        self.values.push(QueryValue::PartitionKey(model.partition_key_values()));
        self
    }

    pub fn append_deletes_by_partition_key(&mut self, iter: &'a [M]) -> &mut Self {
        for model in iter {
            self.append_delete_by_partition_key(model);
        }
        self
    }

    pub fn append_statement(&mut self, statement: &str, val: Val) -> &mut Self {
        self.append_query_to_batch(statement);
        self.values.push(QueryValue::Owned(val));
        self
    }

    pub async fn execute(&self, db_session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        let result = db_session
            .batch(&self.inner, &self.values)
            .await
            .map_err(|e| CharybdisError::BatchError(M::DB_MODEL_NAME, e))?;

        Ok(result)
    }
}

impl<'a, Val: SerializeRow, M: Model> Default for CharybdisModelBatch<'a, Val, M> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ModelBatch<'a>: Model {
    fn batch() -> CharybdisModelBatch<'a, Self, Self> {
        CharybdisModelBatch::new()
    }

    fn unlogged_batch() -> CharybdisModelBatch<'a, Self, Self> {
        CharybdisModelBatch::unlogged()
    }

    fn primary_key_batch() -> CharybdisModelBatch<'a, Self::PrimaryKey, Self> {
        CharybdisModelBatch::new()
    }

    fn unlogged_primary_key_batch() -> CharybdisModelBatch<'a, Self::PrimaryKey, Self> {
        CharybdisModelBatch::unlogged()
    }

    fn delete_batch() -> CharybdisModelBatch<'a, Self::PrimaryKey, Self> {
        CharybdisModelBatch::new()
    }

    fn unlogged_delete_batch() -> CharybdisModelBatch<'a, Self::PrimaryKey, Self> {
        CharybdisModelBatch::unlogged()
    }

    fn partition_key_batch() -> CharybdisModelBatch<'a, Self::PartitionKey, Self> {
        CharybdisModelBatch::new()
    }

    fn unlogged_partition_key_batch() -> CharybdisModelBatch<'a, Self::PartitionKey, Self> {
        CharybdisModelBatch::unlogged()
    }

    fn statement_batch<Val: SerializeRow>() -> CharybdisModelBatch<'a, Val, Self> {
        CharybdisModelBatch::new()
    }

    fn unlogged_statement_batch<Val: SerializeRow>() -> CharybdisModelBatch<'a, Val, Self> {
        CharybdisModelBatch::unlogged()
    }
}

impl<M: Model> ModelBatch<'_> for M {}

pub struct CharybdisBatch<'a> {
    inner: Batch,
    values: Vec<Box<dyn SerializeRow + 'a>>,
}

impl<'a> CharybdisBatch<'a> {
    pub fn new() -> Self {
        Self {
            inner: Batch::default(),
            values: Vec::new(),
        }
    }

    pub fn unlogged() -> Self {
        Self {
            inner: Batch::new(BatchType::Unlogged),
            values: Vec::new(),
        }
    }

    pub fn append<Val: SerializeRow, M: Model, RtQe: QueryExecutor>(
        &mut self,
        query: CharybdisQuery<'a, Val, M, RtQe>,
    ) {
        self.inner.append_statement(query.contents().as_str());
        self.values.push(Box::new(query.values));
    }
}
