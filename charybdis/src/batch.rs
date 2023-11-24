use crate::errors::CharybdisError;
use crate::model::Model;
use crate::{SerializedValues, ValueList};
use scylla::{CachingSession, QueryResult};

// Simple batch for Charybdis models
pub struct CharybdisModelBatch {
    batch: scylla::batch::Batch,
    values: Vec<SerializedValues>,
}

impl CharybdisModelBatch {
    pub fn new() -> Self {
        Self {
            batch: scylla::batch::Batch::default(),
            values: Vec::new(),
        }
    }

    pub fn unlogged() -> Self {
        Self {
            batch: scylla::batch::Batch::new(scylla::batch::BatchType::Unlogged),
            values: Vec::new(),
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

    fn append_statement_to_batch(&mut self, statement: &str) {
        let query = statement.to_string();

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
