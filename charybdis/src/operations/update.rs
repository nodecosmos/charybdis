use crate::callbacks::{Callbacks, ExtCallbacks};
use crate::errors::CharybdisError;
use crate::model::Model;
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

pub trait UpdateWithCallbacks<T: Update + Callbacks> {
    async fn update_cb(&mut self, session: &CachingSession) -> Result<QueryResult, T::Error>;
}

impl<T: Update + Callbacks> UpdateWithCallbacks<T> for T {
    async fn update_cb(&mut self, session: &CachingSession) -> Result<QueryResult, T::Error> {
        self.before_update(session).await?;
        let res = self.update(session).await;
        self.after_update(session).await?;

        res.map_err(T::Error::from)
    }
}

pub trait UpdateWithExtCallbacks<T: Update + ExtCallbacks> {
    async fn update_cb(&mut self, session: &CachingSession, extension: &T::Extension) -> Result<QueryResult, T::Error>;
}

impl<T: Update + ExtCallbacks> UpdateWithExtCallbacks<T> for T {
    async fn update_cb(&mut self, session: &CachingSession, extension: &T::Extension) -> Result<QueryResult, T::Error> {
        self.before_update(session, extension).await?;
        let res = self.update(session).await;
        self.after_update(session, extension).await?;

        res.map_err(T::Error::from)
    }
}
