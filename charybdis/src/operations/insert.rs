use crate::callbacks::{Callbacks, ExtCallbacks};
use crate::errors::CharybdisError;
use crate::model::Model;
use scylla::frame::value::ValueList;
use scylla::{CachingSession, QueryResult};

pub trait Insert: Model + ValueList {
    async fn insert(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError>;
}

impl<T: Model + ValueList> Insert for T {
    async fn insert(&self, session: &CachingSession) -> Result<QueryResult, CharybdisError> {
        session
            .execute(T::INSERT_QUERY, self)
            .await
            .map_err(CharybdisError::from)
    }
}

pub trait InsertWithCallbacks<T: Insert + Callbacks> {
    async fn insert_cb(&mut self, session: &CachingSession) -> Result<QueryResult, T::Error>;
}

impl<T: Insert + Callbacks> InsertWithCallbacks<T> for T {
    async fn insert_cb(&mut self, session: &CachingSession) -> Result<QueryResult, T::Error> {
        self.before_insert(session).await?;
        let res = self.insert(session).await;
        self.after_insert(session).await?;

        res.map_err(T::Error::from)
    }
}

pub trait InsertWithExtCallbacks<T: Insert + ExtCallbacks> {
    async fn insert_cb(&mut self, session: &CachingSession, extension: &T::Extension) -> Result<QueryResult, T::Error>;
}

impl<T: Insert + ExtCallbacks> InsertWithExtCallbacks<T> for T {
    async fn insert_cb(&mut self, session: &CachingSession, extension: &T::Extension) -> Result<QueryResult, T::Error> {
        self.before_insert(session, extension).await?;
        let res = self.insert(session).await;
        self.after_insert(session, extension).await?;

        res.map_err(T::Error::from)
    }
}
