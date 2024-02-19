use crate::errors::CharybdisError;
use crate::model::Model;
use crate::options::{Consistency, ExecutionProfileHandle, RetryPolicy, SerialConsistency};
use crate::query::{CharybdisQuery, QueryExecutor, QueryValue};
use scylla::batch::{Batch, BatchType};
use scylla::history::HistoryListener;
use scylla::serialize::row::SerializeRow;
use scylla::{CachingSession, QueryResult};
use std::sync::Arc;

/// Send + Sync is required for `batch` to be sent to another thread, we need to use `Box` to
/// make it possible to store different types in the same vector. Earlier we used `SerializedValues`
/// and it was possible to store different types in the same vector. With new serialization API
/// we can't do that anymore, so we need to use `Box` to store different types in the same vector.
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
        &self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M, M> = CharybdisModelBatch::new();

            batch.append_inserts(chunk)?;

            db_session.batch(&self.inner, &batch.values).await?;
        }

        Ok(())
    }

    pub async fn chunked_update(
        &self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M, M> = CharybdisModelBatch::new();

            batch.append_updates(chunk)?;

            db_session.batch(&self.inner, &batch.values).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete(
        &self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M::PrimaryKey, M> = CharybdisModelBatch::new();

            for model in chunk {
                batch.append_delete(model)?;
            }

            db_session.batch(&self.inner, &batch.values).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete_by_partition_key(
        &self,
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch: CharybdisModelBatch<M::PartitionKey, M> = CharybdisModelBatch::new();

            batch.append_deletes_by_partition_key(chunk)?;

            db_session.batch(&self.inner, &batch.values).await?;
        }

        Ok(())
    }

    pub async fn chunked_statements(
        &self,
        db_session: &CachingSession,
        statement: &str,
        mut values: Vec<Val>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        while !values.is_empty() {
            let chunk: Vec<Val> = values.drain(..std::cmp::min(chunk_size, values.len())).collect();
            let batch: CharybdisModelBatch<Val, M> = CharybdisModelBatch::new();

            batch.statements(db_session, statement, chunk).await?;

            db_session.batch(&self.inner, &batch.values).await?;
        }

        Ok(())
    }

    pub async fn statements(
        &self,
        db_session: &CachingSession,
        statement: &str,
        values: Vec<Val>,
    ) -> Result<(), CharybdisError> {
        let mut batch: CharybdisModelBatch<Val, M> = CharybdisModelBatch::new();

        for val in values {
            batch.append_statement(statement, val)?;
        }

        db_session.batch(&self.inner, &batch.values).await?;

        Ok(())
    }

    fn append_statement_to_batch(&mut self, statement: &str) {
        let query = statement.to_string();

        self.inner.append_statement(query.as_str());
    }

    pub fn append_insert(&mut self, model: &'a M) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::INSERT_QUERY);

        self.values.push(QueryValue::Model(model));

        Ok(())
    }

    pub fn append_inserts(&mut self, iter: &'a [M]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_insert(model);
            result?
        }

        Ok(())
    }

    pub fn append_update(&mut self, model: &'a M) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::UPDATE_QUERY);

        self.values.push(QueryValue::Model(model));

        Ok(())
    }

    pub fn append_updates(&mut self, iter: &'a [M]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_update(model);
            result?
        }

        Ok(())
    }

    pub fn append_delete(&mut self, model: &M) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::DELETE_QUERY);

        self.values.push(QueryValue::PrimaryKey(model.primary_key_values()));

        Ok(())
    }

    pub fn append_deletes(&mut self, iter: &[M]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_delete(model);
            result?;
        }

        Ok(())
    }

    pub fn append_delete_by_partition_key(&mut self, model: &'a M) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::DELETE_BY_PARTITION_KEY_QUERY);

        self.values.push(QueryValue::PartitionKey(model.partition_key_values()));

        Ok(())
    }

    pub fn append_deletes_by_partition_key(&mut self, iter: &'a [M]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_delete_by_partition_key(model);
            result?
        }

        Ok(())
    }

    pub fn append_statement(&mut self, statement: &str, val: Val) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(statement);

        self.values.push(QueryValue::Owned(val));

        Ok(())
    }

    pub async fn execute(&self, db_session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        let result = db_session.batch(&self.inner, &self.values).await?;

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
