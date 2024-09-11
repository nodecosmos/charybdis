use scylla::_macro_internal::SerializeRow;
use crate::callbacks::{Callbacks, UpdateAction};
use crate::model::{Model};
use crate::query::{CharybdisCbQuery, CharybdisQuery, ModelMutation, QueryValue};

pub trait Update: Model {
    fn update(&self) -> CharybdisQuery<Self, Self, ModelMutation> {
        CharybdisQuery::new(Self::UPDATE_QUERY, QueryValue::Model(self))
    }

    fn update_with_value<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<Val, Self, ModelMutation> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }
}


impl<M: Model> Update for M {}

pub trait UpdateWithCallbacks<'a>: Callbacks {
    fn update_cb(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<'a, Self, UpdateAction<Self>, Self> {
        CharybdisCbQuery::new(Self::UPDATE_QUERY, self, extension)
    }
}

impl<'a, M: Callbacks> UpdateWithCallbacks<'a> for M {}
