use crate::model::BaseModel;
use crate::query::{CharybdisQuery, Paged, QueryValue, RowStream, SingleRow};
use crate::SerializeRow;
use scylla::Bytes;

/// Configurable Find Queries
pub trait Find: BaseModel {
    fn find<'a, Val: SerializeRow>(query: &'static str, values: Val) -> CharybdisQuery<'a, Val, Self, RowStream<Self>>;

    fn find_paged<Val: SerializeRow>(
        query: &'static str,
        values: Val,
        paging_state: Option<Bytes>,
    ) -> CharybdisQuery<Val, Self, Paged<Self>>;

    fn find_first<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<'a, Val, Self, SingleRow<Self>>;

    fn find_by_primary_key_value(value: &Self::PrimaryKey) -> CharybdisQuery<Self::PrimaryKey, Self, SingleRow<Self>>;

    fn find_by_partition_key_value(
        value: &Self::PartitionKey,
    ) -> CharybdisQuery<Self::PartitionKey, Self, RowStream<Self>>;

    fn find_first_by_partition_key_value(
        value: &Self::PartitionKey,
    ) -> CharybdisQuery<Self::PartitionKey, Self, SingleRow<Self>>;

    fn find_by_primary_key(&self) -> CharybdisQuery<Self::PrimaryKey, Self, SingleRow<Self>>;

    fn find_by_partition_key(&self) -> CharybdisQuery<Self::PartitionKey, Self, RowStream<Self>>;
}

impl<M: BaseModel> Find for M {
    fn find<'a, Val: SerializeRow>(query: &'static str, values: Val) -> CharybdisQuery<'a, Val, Self, RowStream<Self>> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn find_paged<Val: SerializeRow>(
        query: &'static str,
        values: Val,
        paging_state: Option<Bytes>,
    ) -> CharybdisQuery<Val, Self, Paged<Self>> {
        CharybdisQuery::new(query, QueryValue::Owned(values)).paging_state(paging_state)
    }

    fn find_first<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<'a, Val, Self, SingleRow<Self>> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn find_by_primary_key_value(value: &Self::PrimaryKey) -> CharybdisQuery<Self::PrimaryKey, M, SingleRow<M>> {
        CharybdisQuery::new(Self::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_by_partition_key_value(
        value: &Self::PartitionKey,
    ) -> CharybdisQuery<Self::PartitionKey, Self, RowStream<Self>> {
        CharybdisQuery::new(Self::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_first_by_partition_key_value(
        value: &Self::PartitionKey,
    ) -> CharybdisQuery<Self::PartitionKey, Self, SingleRow<Self>> {
        CharybdisQuery::new(Self::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_by_primary_key(&self) -> CharybdisQuery<Self::PrimaryKey, Self, SingleRow<Self>> {
        CharybdisQuery::new(
            Self::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    fn find_by_partition_key(&self) -> CharybdisQuery<Self::PartitionKey, Self, RowStream<Self>> {
        CharybdisQuery::new(
            Self::FIND_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }
}
