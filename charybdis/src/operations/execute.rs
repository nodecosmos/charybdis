use crate::errors::CharybdisError;
use crate::scylla::PagingState;
use scylla::client::caching_session::CachingSession;
use scylla::client::pager::QueryPager;
use scylla::response::query_result::QueryResult;
use scylla::response::PagingStateResponse;
use scylla::serialize::row::SerializeRow;

pub async fn execute_unpaged(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
) -> Result<QueryResult, CharybdisError> {
    let res = session
        .execute_unpaged(query, values)
        .await
        .map_err(|e| CharybdisError::ExecutionError(query, e))?;

    Ok(res)
}

pub async fn execute_iter(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
) -> Result<QueryPager, CharybdisError> {
    let res = session
        .execute_iter(query, values)
        .await
        .map_err(|e| CharybdisError::PagerExecutionError(query, e))?;

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
        .map_err(|e| CharybdisError::ExecutionError(query, e))?;

    Ok(res)
}
