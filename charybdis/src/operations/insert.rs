use crate::callbacks::{Callbacks, InsertAction};
use crate::model::Model;
use crate::query::{CharybdisCbQuery, CharybdisQuery, ModelMutation, QueryValue};

pub trait Insert: Model {
    fn insert(&self) -> CharybdisQuery<Self, Self, ModelMutation> {
        CharybdisQuery::new(Self::INSERT_QUERY, QueryValue::Model(self))
    }

    fn insert_if_not_exists(&self) -> CharybdisQuery<Self, Self, ModelMutation> {
        CharybdisQuery::new(Self::INSERT_IF_NOT_EXIST_QUERY, QueryValue::Model(self))
    }
}

impl<M: Model> Insert for M {}

pub trait InsertWithCallbacks<'a>: Callbacks {
    fn insert_cb(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<'a, Self, InsertAction<Self>, Self> {
        CharybdisCbQuery::new(Self::INSERT_QUERY, self, extension)
    }
}

impl<'a, M: Callbacks> InsertWithCallbacks<'a> for M {}
