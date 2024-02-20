use crate::errors::CharybdisError;
use crate::model::Model;
use crate::query::QueryValue;
use crate::SerializeRow;
use scylla::CachingSession;

/// Callbacks are simple trait that can be implemented to add custom logic to the
/// insert, update and delete operations. It's a way to wrap business logic in models.
/// Usually, `before_<action>` callbacks are used to validate the data and set default values, while
/// `after_<action>` callbacks are used to perform additional async operations, like populating elasticsearch client,
/// sending messages to kafka, etc.
/// In case one doesn't need extension it can be set to `Option<()>` and then
/// it can be set to `None` when calling the operation.
pub trait Callbacks: Model {
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

// The compiler issues warnings about potential cycles in the code due to callbacks when attempting to associate
// operations with callbacks in CharybdisCbQuery::execute.
// To circumvent these warnings and clearly delineate callback actions,
// it's necessary to employ distinct structs for each callback operation.
pub struct InsertAction<M: Callbacks>(M);
pub struct UpdateAction<M: Callbacks>(M);
pub struct DeleteAction<M: Callbacks>(M);

pub trait CallbackAction<M: Callbacks> {
    fn query_value<Val: SerializeRow>(model: &M) -> QueryValue<Val, M>;

    async fn before_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error>;

    async fn after_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error>;
}

impl<M: Callbacks> CallbackAction<M> for InsertAction<M> {
    fn query_value<Val: SerializeRow>(model: &M) -> QueryValue<Val, M> {
        QueryValue::Model(&model)
    }

    async fn before_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error> {
        model.before_insert(session, extension).await
    }

    async fn after_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error> {
        model.after_insert(session, extension).await
    }
}

impl<M: Callbacks> CallbackAction<M> for UpdateAction<M> {
    fn query_value<Val: SerializeRow>(model: &M) -> QueryValue<Val, M> {
        QueryValue::Model(&model)
    }

    async fn before_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error> {
        model.before_update(session, extension).await
    }

    async fn after_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error> {
        model.after_update(session, extension).await
    }
}

impl<M: Callbacks> CallbackAction<M> for DeleteAction<M> {
    fn query_value<Val: SerializeRow>(model: &M) -> QueryValue<Val, M> {
        QueryValue::PrimaryKey(model.primary_key_values())
    }

    async fn before_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error> {
        model.before_delete(session, extension).await
    }

    async fn after_execute(model: &mut M, session: &CachingSession, extension: &M::Extension) -> Result<(), M::Error> {
        model.after_delete(session, extension).await
    }
}
