use crate::callbacks::{Callbacks, DeleteAction};
use crate::model::Model;
use crate::query::{CharybdisCbQuery, CharybdisQuery, ModelMutation, QueryValue};
use crate::SerializeRow;

pub trait Delete: Model {
    fn delete_by_query<Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<Val, Self, ModelMutation> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

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
    ) -> CharybdisCbQuery<Self, DeleteAction<Self>, Self::PrimaryKey> {
        CharybdisCbQuery::new(Self::DELETE_QUERY, self, extension)
    }
}

impl<'a, M: Callbacks> DeleteWithCallbacks<'a> for M {}
