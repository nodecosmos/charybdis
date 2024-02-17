use crate::callbacks::Callbacks;
use crate::model::Model;
use crate::operations::OperationsWithCallbacks;
use crate::query::{CharybdisCbQuery, CharybdisQuery, QueryResultWrapper, QueryValue};

pub trait Update: Model {
    fn update(&self) -> CharybdisQuery<Self, Self, QueryResultWrapper>;
}

impl<M: Model> Update for M {
    fn update(&self) -> CharybdisQuery<Self, Self, QueryResultWrapper> {
        CharybdisQuery::new(Self::UPDATE_QUERY, QueryValue::Ref(self))
    }
}

pub trait UpdateWithCallbacks: Update + Callbacks {
    fn update_cb<'a>(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<Self, Self>;
}

impl<M: Model + Callbacks> UpdateWithCallbacks for M {
    fn update_cb<'a>(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<Self, Self> {
        CharybdisCbQuery::new(Self::UPDATE_QUERY, OperationsWithCallbacks::Update, extension, self)
    }
}
