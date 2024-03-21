use crate::errors::CharybdisError;
use crate::SerializeRow;
use scylla::query::Query;
use scylla::{CachingSession, QueryResult};

pub async fn execute(
    session: &CachingSession,
    query: impl Into<Query>,
    values: impl SerializeRow,
) -> Result<QueryResult, CharybdisError> {
    let contents = query.into().contents;

    let res = session
        .execute(contents.clone(), values)
        .await
        .map_err(|e| CharybdisError::QueryError(contents, e))?;
    Ok(res)
}
