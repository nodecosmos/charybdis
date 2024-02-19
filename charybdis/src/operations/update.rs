use crate::callbacks::{Callbacks, CbModel, UpdateCbModel};
use crate::model::Model;
use crate::query::{CharybdisCbQuery, CharybdisQuery, ModelMutation, QueryValue};

pub trait Update: Model {
    fn update(&self) -> CharybdisQuery<Self, Self, ModelMutation> {
        CharybdisQuery::new(Self::UPDATE_QUERY, QueryValue::Ref(self))
    }
}

impl<M: Model> Update for M {}

pub trait UpdateWithCallbacks<'a>: Callbacks {
    fn update_cb(
        &'a mut self,
        extension: &'a Self::Extension,
    ) -> CharybdisCbQuery<'a, Self, UpdateCbModel<Self>, Self::PrimaryKey> {
        CharybdisCbQuery::new(Self::UPDATE_QUERY, UpdateCbModel::new(self, extension))
    }
}

impl<'a, M: Callbacks> UpdateWithCallbacks<'a> for M {}
