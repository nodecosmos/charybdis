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

/// Compiler complains about possible cycles in the code when using callbacks if we try to match operation with
/// callback within CharybdisCbQuery::execute, so we need to use separate structs for each callback operation in order to clearly
/// separate between actions and avoid warnings.
pub struct InsertCbModel<'a, M: Callbacks> {
    model: &'a mut M,
    extension: &'a M::Extension,
}

pub struct UpdateCbModel<'a, M: Callbacks> {
    model: &'a mut M,
    extension: &'a M::Extension,
}

pub struct DeleteCbModel<'a, M: Callbacks> {
    model: &'a mut M,
    extension: &'a M::Extension,
}

pub trait CbModel<'a, M: Callbacks> {
    type Error: From<CharybdisError>;

    fn new(model: &'a mut M, extension: &'a M::Extension) -> Self;

    fn value<Val: SerializeRow>(&self) -> QueryValue<Val, M>;

    async fn before_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error>;

    async fn after_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error>;
}

impl<'a, M: Callbacks> CbModel<'a, M> for InsertCbModel<'a, M> {
    type Error = M::Error;

    fn new(model: &'a mut M, extension: &'a M::Extension) -> Self {
        InsertCbModel { model, extension }
    }

    fn value<Val: SerializeRow>(&self) -> QueryValue<Val, M> {
        QueryValue::Model(&self.model)
    }

    async fn before_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error> {
        self.model.before_insert(session, self.extension).await
    }

    async fn after_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error> {
        self.model.after_insert(session, self.extension).await
    }
}

impl<'a, M: Callbacks> CbModel<'a, M> for UpdateCbModel<'a, M> {
    type Error = M::Error;

    fn new(model: &'a mut M, extension: &'a M::Extension) -> Self {
        UpdateCbModel { model, extension }
    }

    fn value<Val: SerializeRow>(&self) -> QueryValue<Val, M> {
        QueryValue::Model(&self.model)
    }

    async fn before_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error> {
        self.model.before_update(session, self.extension).await
    }

    async fn after_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error> {
        self.model.after_update(session, self.extension).await
    }
}

impl<'a, M: Callbacks> CbModel<'a, M> for DeleteCbModel<'a, M> {
    type Error = M::Error;

    fn new(model: &'a mut M, extension: &'a M::Extension) -> Self {
        DeleteCbModel { model, extension }
    }

    fn value<Val: SerializeRow>(&self) -> QueryValue<Val, M> {
        QueryValue::PrimaryKey(self.model.primary_key_values())
    }

    async fn before_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error> {
        self.model.before_delete(session, self.extension).await
    }

    async fn after_execute(&mut self, session: &CachingSession) -> Result<(), Self::Error> {
        self.model.after_delete(session, self.extension).await
    }
}
