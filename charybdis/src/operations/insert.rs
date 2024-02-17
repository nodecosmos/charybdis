use crate::callbacks::Callbacks;
use crate::model::Model;
use crate::operations::OperationsWithCallbacks;
use crate::query::{CharybdisCbQuery, CharybdisQuery, QueryResultWrapper, QueryValue};

pub trait Insert: Model {
    fn insert(&self) -> CharybdisQuery<Self, Self, QueryResultWrapper>;
    fn insert_if_not_exists(&self) -> CharybdisQuery<Self, Self, QueryResultWrapper>;
}

impl<M: Model> Insert for M {
    fn insert(&self) -> CharybdisQuery<Self, Self, QueryResultWrapper> {
        CharybdisQuery::new(Self::INSERT_QUERY, QueryValue::Ref(self))
    }

    fn insert_if_not_exists(&self) -> CharybdisQuery<Self, Self, QueryResultWrapper> {
        CharybdisQuery::new(Self::INSERT_IF_NOT_EXIST_QUERY, QueryValue::Ref(self))
    }
}

pub trait InsertWithCallbacks<'a>: Insert + Callbacks {
    fn insert_cb(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<'a, Self, Self>;
}

impl<'a, M: Model + Callbacks> InsertWithCallbacks<'a> for M {
    fn insert_cb(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<'a, Self, Self> {
        CharybdisCbQuery::new(Self::INSERT_QUERY, OperationsWithCallbacks::Insert, extension, self)
    }
}
