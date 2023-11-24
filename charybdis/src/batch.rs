use crate::errors::CharybdisError;
use crate::model::Model;
use crate::{SerializedValues, ValueList};
use scylla::{CachingSession, QueryResult};

// Simple batch for Charybdis models
pub struct CharybdisModelBatch {
    batch: scylla::batch::Batch,
    values: Vec<SerializedValues>,
    with_unique_timestamp: bool,
    current_timestamp: i64,
}

impl CharybdisModelBatch {
    pub fn new() -> Self {
        Self {
            batch: scylla::batch::Batch::default(),
            values: Vec::new(),
            with_unique_timestamp: false,
            current_timestamp: chrono::Utc::now().timestamp_micros(),
        }
    }

    pub fn unlogged() -> Self {
        Self {
            batch: scylla::batch::Batch::new(scylla::batch::BatchType::Unlogged),
            values: Vec::new(),
            with_unique_timestamp: false,
            current_timestamp: chrono::Utc::now().timestamp_micros(),
        }
    }

    pub async fn chunked_insert<T: Model + ValueList>(
        db_session: &CachingSession,
        iter: &Vec<T>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch = Self::unlogged();

            batch.append_inserts(chunk)?;

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_update<T: Model + ValueList>(
        db_session: &CachingSession,
        iter: &Vec<T>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch = Self::unlogged();

            batch.append_updates(chunk)?;

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete<T: Model + ValueList>(
        db_session: &CachingSession,
        iter: &Vec<T>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch = Self::unlogged();

            for model in chunk {
                batch.append_delete(model)?;
            }

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete_by_partition_key<T: Model + ValueList>(
        db_session: &CachingSession,
        iter: &Vec<T>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch = Self::unlogged();

            batch.append_deletes_by_partition_key(chunk)?;

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub fn with_unique_timestamp(mut self) -> Self {
        self.with_unique_timestamp = true;
        self
    }

    fn inject_timestamp(&mut self, statement: &str) -> String {
        return if statement.contains("SET") {
            // insert timestamp before SET
            let mut parts = statement.split("SET");
            let first_part = parts.next().unwrap_or_default();
            let second_part = parts.next().unwrap_or_default();
            format!(
                "{} USING TIMESTAMP {} SET{}",
                first_part, self.current_timestamp, second_part
            )
        } else if statement.contains("DELETE") {
            // insert timestamp before WHERE
            let mut parts = statement.split("WHERE");
            let first_part = parts.next().unwrap_or_default();
            let second_part = parts.next().unwrap_or_default();

            format!(
                "{} USING TIMESTAMP {} WHERE{}",
                first_part, self.current_timestamp, second_part
            )
        } else {
            // append timestamp to the end
            format!("{} USING TIMESTAMP {}", statement, self.current_timestamp)
        };
    }

    fn append_statement_to_batch(&mut self, statement: &str) {
        let mut query = statement.to_string();

        if self.with_unique_timestamp {
            query = self.inject_timestamp(statement);
            self.current_timestamp += 1;
        }

        self.batch.append_statement(query.as_str());
    }

    pub fn append_insert<T: Model + ValueList>(&mut self, model: &T) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(T::INSERT_QUERY);
        let values = model.serialized()?;

        self.values.push(values.into_owned());

        Ok(())
    }

    pub fn append_inserts<T: Model + ValueList>(&mut self, iter: &[T]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_insert(model);
            result?
        }

        Ok(())
    }

    pub fn append_update<T: Model>(&mut self, model: &T) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(T::UPDATE_QUERY);

        let update_values = model
            .update_values()
            .map_err(|e| CharybdisError::SerializeValuesError(e, T::DB_MODEL_NAME.to_string()))?;

        self.values.push(update_values.into_owned());

        Ok(())
    }

    pub fn append_updates<T: Model + ValueList>(&mut self, iter: &[T]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_update(model);
            result?
        }

        Ok(())
    }

    pub fn append_delete<T: Model + ValueList>(&mut self, model: &T) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(T::DELETE_QUERY);

        let primary_key_values = model
            .primary_key_values()
            .map_err(|e| CharybdisError::SerializeValuesError(e, T::DB_MODEL_NAME.to_string()))?;

        self.values.push(primary_key_values.into_owned());

        Ok(())
    }

    pub fn append_deletes<I, T>(&mut self, iter: I) -> Result<(), CharybdisError>
    where
        I: Iterator<Item = T>,
        T: Model + ValueList,
    {
        for model in iter {
            let result = self.append_delete(&model);
            result?;
        }

        Ok(())
    }

    pub fn append_delete_by_partition_key<T: Model + ValueList>(&mut self, model: &T) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(T::DELETE_BY_PARTITION_KEY_QUERY);

        let partition_key_values = model
            .partition_key_values()
            .map_err(|e| CharybdisError::SerializeValuesError(e, T::DB_MODEL_NAME.to_string()))?;

        self.values.push(partition_key_values.into_owned());

        Ok(())
    }

    pub fn append_deletes_by_partition_key<T: Model + ValueList>(&mut self, iter: &[T]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_delete_by_partition_key(model);
            result?
        }

        Ok(())
    }

    pub fn append_statement(&mut self, statement: &str, values: impl ValueList) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(statement);

        let values = values.serialized()?;

        self.values.push(values.into_owned());

        Ok(())
    }

    pub async fn execute(&self, db_session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        let result = db_session.batch(&self.batch, &self.values).await?;

        Ok(result)
    }
}

impl Default for CharybdisModelBatch {
    fn default() -> Self {
        Self::new()
    }
}
