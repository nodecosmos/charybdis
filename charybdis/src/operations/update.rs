use crate::callbacks::Callbacks;
use crate::errors::CharybdisError;
use crate::model::Model;
use crate::operations::OperationsWithCallbacks;
use crate::query::CharybdisCbQuery;
use scylla::{CachingSession, QueryResult};

pub trait Update {
    async fn update(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError>;
}

impl<T: Model> Update for T {
    async fn update(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        session
            .execute(Self::UPDATE_QUERY, self)
            .await
            .map_err(CharybdisError::from)
    }
}

pub trait UpdateWithCallbacks<'a, M: Model + Callbacks> {
    fn update_cb(&'a mut self, extension: &'a M::Extension) -> CharybdisCbQuery<'a, M, M>;
}

impl<'a, M: Model + Callbacks> UpdateWithCallbacks<'a, M> for M {
    fn update_cb(&'a mut self, extension: &'a M::Extension) -> CharybdisCbQuery<'a, M, M> {
        CharybdisCbQuery::new(Self::UPDATE_QUERY, OperationsWithCallbacks::Update, extension, self)
    }
}
