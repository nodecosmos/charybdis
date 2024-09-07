use crate::errors::CharybdisError;
use crate::scylla::PagingState;
use scylla::serialize::row::SerializeRow;
use scylla::statement::PagingStateResponse;
use scylla::transport::iterator::RowIterator;
use scylla::{CachingSession, QueryResult};

pub async fn execute_unpaged(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
) -> Result<QueryResult, CharybdisError> {
    let res = session
        .execute_unpaged(query, values)
        .await
        .map_err(|e| CharybdisError::QueryError(query, e))?;

    Ok(res)
}

pub async fn execute_iter(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
) -> Result<RowIterator, CharybdisError> {
    let res = session
        .execute_iter(query, values)
        .await
        .map_err(|e| CharybdisError::QueryError(query, e))?;

    Ok(res)
}

pub async fn execute_single_page(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
    paging_state: PagingState,
) -> Result<(QueryResult, PagingStateResponse), CharybdisError> {
    let res = session
        .execute_single_page(query, values, paging_state)
        .await
        .map_err(|e| CharybdisError::QueryError(query, e))?;

    Ok(res)
}
