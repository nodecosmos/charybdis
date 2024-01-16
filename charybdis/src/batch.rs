use crate::errors::CharybdisError;
use crate::iterator::IntoOwnedChunks;
use crate::model::Model;
use scylla::serialize::row::SerializeRow;
use scylla::{CachingSession, QueryResult};

pub struct CharybdisModelBatch<'a> {
    batch: scylla::batch::Batch,
    values: Vec<Box<dyn SerializeRow + Send + Sync + 'a>>,
}

impl<'a> CharybdisModelBatch<'a> {
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

    pub async fn chunked_insert<M: Model + Send + Sync + 'a>(
        db_session: &CachingSession,
        iter: Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.into_owned_chunks(chunk_size);

        for chunk in chunks {
            let mut batch = CharybdisModelBatch::unlogged();

            batch.append_inserts(chunk)?;

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_update<M: Model + Send + Sync + 'a>(
        db_session: &CachingSession,
        iter: Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.into_owned_chunks(chunk_size);

        for chunk in chunks {
            let mut batch = CharybdisModelBatch::unlogged();

            batch.append_updates(chunk)?;

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete<M: Model + Send + Sync + 'a>(
        db_session: &CachingSession,
        iter: &Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.chunks(chunk_size);

        for chunk in chunks {
            let mut batch = CharybdisModelBatch::unlogged();

            for model in chunk {
                batch.append_delete(model)?;
            }

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    pub async fn chunked_delete_by_partition_key<M: Model + Send + Sync + 'a>(
        db_session: &CachingSession,
        iter: Vec<M>,
        chunk_size: usize,
    ) -> Result<(), CharybdisError> {
        let chunks = iter.into_owned_chunks(chunk_size);

        for chunk in chunks {
            let mut batch = CharybdisModelBatch::unlogged();

            batch.append_deletes_by_partition_key(chunk)?;

            batch.execute(db_session).await?;
        }

        Ok(())
    }

    fn append_statement_to_batch(&mut self, statement: &str) {
        let query = statement.to_string();

        self.batch.append_statement(query.as_str());
    }

    pub fn append_insert<M: Model + Send + Sync + 'a>(&mut self, model: M) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::INSERT_QUERY);

        self.values.push(Box::new(model));

        Ok(())
    }

    pub fn append_inserts<M: Model + Send + Sync + 'a>(&mut self, iter: Vec<M>) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_insert::<M>(model);
            result?
        }

        Ok(())
    }

    pub fn append_update<M: Model + Send + Sync + 'a>(&mut self, model: M) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::UPDATE_QUERY);

        self.values.push(Box::new(model));

        Ok(())
    }

    pub fn append_updates<M: Model + Send + Sync + 'a>(&mut self, iter: Vec<M>) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_update(model);
            result?
        }

        Ok(())
    }

    pub fn append_delete<M: Model + Send + Sync + 'a>(&mut self, model: &M) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::DELETE_QUERY);

        self.values.push(Box::new(model.primary_key_values()));

        Ok(())
    }

    pub fn append_deletes<M: Model + Send + Sync + 'a>(&mut self, iter: &[M]) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_delete(model);
            result?;
        }

        Ok(())
    }

    pub fn append_delete_by_partition_key<M: Model + Send + Sync + 'a>(
        &mut self,
        model: M,
    ) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(M::DELETE_BY_PARTITION_KEY_QUERY);

        self.values.push(Box::new(model.partition_key_values()));

        Ok(())
    }

    pub fn append_deletes_by_partition_key<M: Model + Send + Sync + 'a>(
        &mut self,
        iter: Vec<M>,
    ) -> Result<(), CharybdisError> {
        for model in iter {
            let result = self.append_delete_by_partition_key(model);
            result?
        }

        Ok(())
    }

    pub fn append_statement<V: SerializeRow + Send + Sync + 'a>(
        &mut self,
        statement: &str,
        val: V,
    ) -> Result<(), CharybdisError> {
        self.append_statement_to_batch(statement);

        self.values.push(Box::new(val));

        Ok(())
    }

    pub async fn execute(&self, db_session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        let mut values = Vec::new();
        for value in &self.values {
            values.push(value.as_ref());
        }

        let result = db_session.batch(&self.batch, &values).await?;

        Ok(result)
    }
}

impl<'a> Default for CharybdisModelBatch<'a> {
    fn default() -> Self {
        Self::new()
    }
}
