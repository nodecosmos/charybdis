use crate::errors::CharybdisError;
use scylla::CachingSession;

pub trait Callbacks {
    type Error: From<CharybdisError>;

    async fn before_insert(&mut self, _session: &CachingSession) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn after_insert(&mut self, _session: &CachingSession) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn before_update(&mut self, _session: &CachingSession) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn after_update(&mut self, _session: &CachingSession) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn before_delete(&mut self, _session: &CachingSession) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn after_delete(&mut self, _session: &CachingSession) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub trait ExtCallbacks {
    type Extension;
    type Error: From<CharybdisError>;

    async fn before_insert(
        &mut self,
        _session: &CachingSession,
        _extension: &Self::Extension,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn after_insert(
        &mut self,
        _session: &CachingSession,
        _extension: &Self::Extension,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn before_update(
        &mut self,
        _session: &CachingSession,
        _extension: &Self::Extension,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn after_update(
        &mut self,
        _session: &CachingSession,
        _extension: &Self::Extension,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn before_delete(
        &mut self,
        _session: &CachingSession,
        _extension: &Self::Extension,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    async fn after_delete(
        &mut self,
        _session: &CachingSession,
        _extension: &Self::Extension,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
