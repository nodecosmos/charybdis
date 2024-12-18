use crate::model::BaseModel;
use crate::query::{CharybdisQuery, ModelPaged, ModelRow, ModelStream, OptionalModelRow, QueryValue};
use scylla::serialize::row::SerializeRow;
use scylla::statement::PagingState;

/// Configurable Find Queries
pub trait Find: BaseModel
where
    Self: 'static,
{
    fn find<'a, Val: SerializeRow>(query: &'static str, values: Val) -> CharybdisQuery<'a, Val, Self, ModelStream> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn find_paged<Val: SerializeRow>(
        query: &'static str,
        values: Val,
        paging_state: PagingState,
    ) -> CharybdisQuery<'static, Val, Self, ModelPaged> {
        CharybdisQuery::new(query, QueryValue::Owned(values)).paging_state(paging_state)
    }

    fn find_first<'a, Val: SerializeRow>(query: &'static str, values: Val) -> CharybdisQuery<'a, Val, Self, ModelRow> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn maybe_find_first<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> CharybdisQuery<'a, Val, Self, OptionalModelRow> {
        CharybdisQuery::new(query, QueryValue::Owned(values))
    }

    fn find_all<'a>() -> CharybdisQuery<'a, (), Self, ModelStream> {
        CharybdisQuery::new(Self::FIND_ALL_QUERY, QueryValue::Empty)
    }

    fn find_by_primary_key_value<'a>(value: Self::PrimaryKey) -> CharybdisQuery<'a, Self::PrimaryKey, Self, ModelRow> {
        CharybdisQuery::new(Self::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Owned(value))
    }

    fn maybe_find_by_primary_key_value<'a>(
        value: Self::PrimaryKey,
    ) -> CharybdisQuery<'a, Self::PrimaryKey, Self, OptionalModelRow> {
        CharybdisQuery::new(Self::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Owned(value))
    }

    fn find_by_partition_key_value<'a>(
        value: Self::PartitionKey,
    ) -> CharybdisQuery<'a, Self::PartitionKey, Self, ModelStream> {
        CharybdisQuery::new(Self::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Owned(value))
    }

    fn find_first_by_partition_key_value<'a>(
        value: Self::PartitionKey,
    ) -> CharybdisQuery<'a, Self::PartitionKey, Self, ModelRow> {
        CharybdisQuery::new(Self::FIND_FIRST_BY_PARTITION_KEY_QUERY, QueryValue::Owned(value))
    }

    fn find_by_partition_key_value_paged<'a>(
        value: Self::PartitionKey,
    ) -> CharybdisQuery<'a, Self::PartitionKey, Self, ModelPaged> {
        CharybdisQuery::new(Self::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Owned(value))
    }

    fn find_by_primary_key(&self) -> CharybdisQuery<Self::PrimaryKey, Self, ModelRow> {
        CharybdisQuery::new(
            Self::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    fn maybe_find_by_primary_key(&self) -> CharybdisQuery<Self::PrimaryKey, Self, OptionalModelRow> {
        CharybdisQuery::new(
            Self::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    fn find_by_partition_key(&self) -> CharybdisQuery<Self::PartitionKey, Self, ModelStream> {
        CharybdisQuery::new(
            Self::FIND_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }
}

impl<M: BaseModel + 'static> Find for M {}
