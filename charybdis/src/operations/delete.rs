use crate::callbacks::{Callbacks, CbModel, DeleteCbModel};
use crate::model::Model;
use crate::query::{CharybdisCbQuery, CharybdisQuery, ModelMutation, QueryValue};

pub trait Delete: Model {
    fn delete(&self) -> CharybdisQuery<Self::PrimaryKey, Self, ModelMutation> {
        CharybdisQuery::new(Self::DELETE_QUERY, QueryValue::Owned(self.primary_key_values()))
    }

    fn delete_by_partition_key(&self) -> CharybdisQuery<Self::PartitionKey, Self, ModelMutation> {
        CharybdisQuery::new(
            Self::DELETE_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }
}

impl<M: Model> Delete for M {}

pub trait DeleteWithCallbacks<'a>: Callbacks {
    fn delete_cb(
        &'a mut self,
        extension: &'a Self::Extension,
    ) -> CharybdisCbQuery<Self, DeleteCbModel<Self>, Self::PrimaryKey> {
        CharybdisCbQuery::new(Self::DELETE_QUERY, DeleteCbModel::new(self, extension))
    }
}

impl<'a, M: Callbacks> DeleteWithCallbacks<'a> for M {}
