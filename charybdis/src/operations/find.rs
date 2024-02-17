use crate::model::BaseModel;
use crate::query::{CharybdisQuery, Paged, QueryValue, RowStream, SingleRow};
use crate::SerializeRow;
use scylla::Bytes;

/// Configurable Find Queries
pub trait Find<'a, Val: SerializeRow, M: BaseModel>: BaseModel {
    fn find(query: &'static str, values: &'a Val) -> CharybdisQuery<'a, Val, M, RowStream<M>>;

    fn find_paged(
        query: &'static str,
        values: &'a Val,
        paging_state: Option<Bytes>,
    ) -> CharybdisQuery<'a, Val, M, Paged<M>>;

    fn find_first(query: &'static str, values: &'a Val) -> CharybdisQuery<'a, Val, M, SingleRow<M>>;

    fn find_by_primary_key_value(value: &'a M::PrimaryKey) -> CharybdisQuery<'a, M::PrimaryKey, M, SingleRow<M>>;

    fn find_by_partition_key_value(value: &'a M::PartitionKey) -> CharybdisQuery<'a, M::PartitionKey, M, RowStream<M>>;

    fn find_first_by_partition_key_value(
        value: &'a M::PartitionKey,
    ) -> CharybdisQuery<'a, M::PartitionKey, M, SingleRow<M>>;

    fn find_by_primary_key(&self) -> CharybdisQuery<M::PrimaryKey, M, SingleRow<M>>;

    fn find_by_partition_key(&self) -> CharybdisQuery<M::PartitionKey, M, RowStream<M>>;
}

impl<'a, Val: SerializeRow, M: BaseModel> Find<'a, Val, M> for M {
    fn find(query: &'static str, values: &'a Val) -> CharybdisQuery<'a, Val, M, RowStream<M>> {
        CharybdisQuery::new(query, QueryValue::Ref(values))
    }

    fn find_paged(
        query: &'static str,
        values: &'a Val,
        paging_state: Option<Bytes>,
    ) -> CharybdisQuery<'a, Val, M, Paged<M>> {
        CharybdisQuery::new(query, QueryValue::Ref(values)).paging_state(paging_state)
    }

    fn find_first(query: &'static str, values: &'a Val) -> CharybdisQuery<'a, Val, M, SingleRow<M>> {
        CharybdisQuery::new(query, QueryValue::Ref(values))
    }

    fn find_by_primary_key_value(value: &'a M::PrimaryKey) -> CharybdisQuery<'a, M::PrimaryKey, M, SingleRow<M>> {
        CharybdisQuery::new(M::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_by_partition_key_value(value: &'a M::PartitionKey) -> CharybdisQuery<'a, M::PartitionKey, M, RowStream<M>> {
        CharybdisQuery::new(M::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_first_by_partition_key_value(
        value: &'a M::PartitionKey,
    ) -> CharybdisQuery<'a, M::PartitionKey, M, SingleRow<M>> {
        CharybdisQuery::new(M::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Ref(value))
    }

    fn find_by_primary_key(&self) -> CharybdisQuery<M::PrimaryKey, M, SingleRow<M>> {
        CharybdisQuery::new(
            M::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    fn find_by_partition_key(&self) -> CharybdisQuery<M::PartitionKey, M, RowStream<M>> {
        CharybdisQuery::new(
            M::FIND_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }
}
