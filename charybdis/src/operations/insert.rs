use crate::callbacks::Callbacks;
use crate::model::Model;
use crate::operations::OperationsWithCallbacks;
use crate::query::{CharybdisCbQuery, CharybdisQuery, QueryResultWrapper, QueryValue};

pub trait Insert<M: Model>: Model {
    fn insert(&self) -> CharybdisQuery<M, M, QueryResultWrapper>;
    fn insert_if_not_exists(&self) -> CharybdisQuery<M, M, QueryResultWrapper>;
}

impl<M: Model> Insert<M> for M {
    fn insert(&self) -> CharybdisQuery<M, M, QueryResultWrapper> {
        CharybdisQuery::new(Self::INSERT_QUERY, QueryValue::Ref(self))
    }

    fn insert_if_not_exists(&self) -> CharybdisQuery<M, M, QueryResultWrapper> {
        CharybdisQuery::new(Self::INSERT_IF_NOT_EXIST_QUERY, QueryValue::Ref(self))
    }
}

pub trait InsertWithCallbacks<'a, M: Model + Callbacks> {
    fn insert_cb(&'a mut self, extension: &'a M::Extension) -> CharybdisCbQuery<'a, M, M>;
}

impl<'a, M: Model + Callbacks> InsertWithCallbacks<'a, M> for M {
    fn insert_cb(&'a mut self, extension: &'a M::Extension) -> CharybdisCbQuery<'a, M, M> {
        CharybdisCbQuery::new(Self::INSERT_QUERY, OperationsWithCallbacks::Insert, extension, self)
    }
}
