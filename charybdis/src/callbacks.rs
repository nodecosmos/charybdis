use crate::errors::CharybdisError;
use scylla::CachingSession;

/// Callbacks are simple trait that can be implemented to add custom logic to the
/// insert, update and delete operations. It's a way to wrap business logic in models.
/// Usually, `before_<action>` callbacks are used to validate the data and set default values, while
/// `after_<action>` callbacks are used to perform additional async operations, like populating elasticsearch client,
/// sending messages to kafka, etc.
/// In case one doesn't need extension it can be set to `Option<()>` and then
/// it can be set to `None` when calling the operation.
pub trait Callbacks {
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
