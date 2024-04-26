use scylla::Bytes;

use crate::model::BaseModel;
use crate::query::{CharybdisQuery, ModelPaged, ModelRow, ModelStream, OptionalModelRow, QueryValue};
use crate::SerializeRow;

/// Configurable Find Queries
pub trait Find: BaseModel {
    fn find<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<'a, Val, Self, ModelStream<Self>> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn find_paged<Val: SerializeRow>(
        query: &'static str,
        values: Val,
        paging_state: Option<Bytes>,
    ) -> CharybdisQuery<Val, Self, ModelPaged<Self>> {
        CharybdisQuery::new(query, QueryValue::Owned(values)).paging_state(paging_state)
    }

    fn find_first<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<'a, Val, Self, ModelRow<Self>> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn maybe_find_first<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<'a, Val, Self, OptionalModelRow<Self>> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn find_by_primary_key_value(value: &Self::PrimaryKey) -> CharybdisQuery<Self::PrimaryKey, Self, ModelRow<Self>> {
        CharybdisQuery::new(Self::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Ref(value))
    }

    fn maybe_find_by_primary_key_value(
        value: &Self::PrimaryKey,
    ) -> CharybdisQuery<Self::PrimaryKey, Self, OptionalModelRow<Self>> {
        CharybdisQuery::new(Self::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_by_partition_key_value(
        value: &Self::PartitionKey,
    ) -> CharybdisQuery<Self::PartitionKey, Self, ModelStream<Self>> {
        CharybdisQuery::new(Self::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_first_by_partition_key_value(
        value: &Self::PartitionKey,
    ) -> CharybdisQuery<Self::PartitionKey, Self, ModelRow<Self>> {
        CharybdisQuery::new(Self::FIND_FIRST_BY_PARTITION_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_by_primary_key(&self) -> CharybdisQuery<Self::PrimaryKey, Self, ModelRow<Self>> {
        CharybdisQuery::new(
            Self::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    fn maybe_find_by_primary_key(&self) -> CharybdisQuery<Self::PrimaryKey, Self, OptionalModelRow<Self>> {
        CharybdisQuery::new(
            Self::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    fn find_by_partition_key(&self) -> CharybdisQuery<Self::PartitionKey, Self, ModelStream<Self>> {
        CharybdisQuery::new(
            Self::FIND_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }
}

impl<M: BaseModel> Find for M {}
