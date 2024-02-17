use crate::callbacks::Callbacks;
use crate::model::Model;
use crate::operations::OperationsWithCallbacks;
use crate::query::{CharybdisCbQuery, CharybdisQuery, QueryResultWrapper, QueryValue};

pub trait Delete: Model {
    fn delete(&self) -> CharybdisQuery<Self::PrimaryKey, Self, QueryResultWrapper>;
    fn delete_by_partition_key(&self) -> CharybdisQuery<Self::PartitionKey, Self, QueryResultWrapper>;
}

impl<M: Model> Delete for M {
    fn delete(&self) -> CharybdisQuery<Self::PrimaryKey, Self, QueryResultWrapper> {
        CharybdisQuery::new(Self::DELETE_QUERY, QueryValue::Owned(self.primary_key_values()))
    }

    fn delete_by_partition_key(&self) -> CharybdisQuery<Self::PartitionKey, Self, QueryResultWrapper> {
        CharybdisQuery::new(
            Self::DELETE_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }
}

pub trait DeleteWithCallbacks<'a>: Model + Callbacks {
    fn delete_cb(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<'a, Self, Self::PrimaryKey>;
}

impl<'a, M: Model + Callbacks> DeleteWithCallbacks<'a> for M {
    fn delete_cb(&'a mut self, extension: &'a Self::Extension) -> CharybdisCbQuery<'a, Self, Self::PrimaryKey> {
        CharybdisCbQuery::new(Self::DELETE_QUERY, OperationsWithCallbacks::Delete, extension, self)
    }
}
