use crate::callbacks::InsertCbModel;
use crate::callbacks::{Callbacks, CbModel};
use crate::model::Model;
use crate::query::{CharybdisCbQuery, CharybdisQuery, ModelMutation, QueryValue};

pub trait Insert: Model {
    fn insert(&self) -> CharybdisQuery<Self, Self, ModelMutation> {
        CharybdisQuery::new(Self::INSERT_QUERY, QueryValue::Ref(self))
    }

    fn insert_if_not_exists(&self) -> CharybdisQuery<Self, Self, ModelMutation> {
        CharybdisQuery::new(Self::INSERT_IF_NOT_EXIST_QUERY, QueryValue::Ref(self))
    }
}

impl<M: Model> Insert for M {}

pub trait InsertWithCallbacks<'a>: Callbacks {
    fn insert_cb(
        &'a mut self,
        extension: &'a Self::Extension,
    ) -> CharybdisCbQuery<'a, Self, InsertCbModel<Self>, Self> {
        CharybdisCbQuery::new(Self::INSERT_QUERY, InsertCbModel::new(self, extension))
    }
}

impl<'a, M: Callbacks> InsertWithCallbacks<'a> for M {}
