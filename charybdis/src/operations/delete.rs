use crate::callbacks::{Callbacks, ExtCallbacks};
use crate::errors::CharybdisError;
use crate::model::Model;
use scylla::frame::value::ValueList;
use scylla::{CachingSession, QueryResult};

pub trait Delete {
    async fn delete(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError>;
    async fn delete_by_partition_key(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError>;
}

impl<T: Model + ValueList> Delete for T {
    async fn delete(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        let primary_key_values = self
            .primary_key_values()
            .map_err(|e| CharybdisError::SerializeValuesError(e, Self::DB_MODEL_NAME.to_string()))?;

        session
            .execute(T::DELETE_QUERY, primary_key_values)
            .await
            .map_err(CharybdisError::QueryError)
    }

    async fn delete_by_partition_key(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        let partition_key_values = self
            .partition_key_values()
            .map_err(|e| CharybdisError::SerializeValuesError(e, Self::DB_MODEL_NAME.to_string()))?;

        session
            .execute(T::DELETE_BY_PARTITION_KEY_QUERY, partition_key_values)
            .await
            .map_err(CharybdisError::QueryError)
    }
}

pub trait DeleteWithCallbacks<T: Delete + Callbacks> {
    async fn delete_cb(&mut self, session: &CachingSession) -> Result<QueryResult, T::Error>;
}

impl<T: Delete + Callbacks> DeleteWithCallbacks<T> for T {
    async fn delete_cb(&mut self, session: &CachingSession) -> Result<QueryResult, T::Error> {
        self.before_delete(session).await?;
        let res = self.delete(session).await;
        self.after_delete(session).await?;

        Ok(res?)
    }
}

pub trait DeleteWithExtCallbacks<T: Delete + ExtCallbacks> {
    async fn delete_cb(&mut self, session: &CachingSession, extension: &T::Extension) -> Result<QueryResult, T::Error>;
}

impl<T: Delete + ExtCallbacks> DeleteWithExtCallbacks<T> for T {
    async fn delete_cb(&mut self, session: &CachingSession, extension: &T::Extension) -> Result<QueryResult, T::Error> {
        self.before_delete(session, extension).await?;
        let res = self.delete(session).await;
        self.after_delete(session, extension).await?;

        Ok(res?)
    }
}
