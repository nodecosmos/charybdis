use crate::callbacks::Callbacks;
use crate::errors::CharybdisError;
use crate::model::Model;
use crate::operations::OperationsWithCallbacks;
use crate::query::CharybdisCbQuery;
use scylla::{CachingSession, QueryResult};

pub trait Delete {
    async fn delete(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError>;
    async fn delete_by_partition_key(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError>;
}

impl<T: Model> Delete for T {
    async fn delete(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        session
            .execute(T::DELETE_QUERY, self.primary_key_values())
            .await
            .map_err(CharybdisError::QueryError)
    }

    async fn delete_by_partition_key(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        session
            .execute(T::DELETE_BY_PARTITION_KEY_QUERY, self.partition_key_values())
            .await
            .map_err(CharybdisError::QueryError)
    }
}

pub trait DeleteWithCallbacks<'a, M: Model + Callbacks> {
    fn delete_cb(&'a mut self, extension: &'a M::Extension) -> CharybdisCbQuery<'a, M, M>;
}

impl<'a, M: Model + Callbacks> DeleteWithCallbacks<'a, M> for M {
    fn delete_cb(&'a mut self, extension: &'a M::Extension) -> CharybdisCbQuery<'a, M, M> {
        CharybdisCbQuery::new(Self::DELETE_QUERY, OperationsWithCallbacks::Delete, extension, self)
    }
}
