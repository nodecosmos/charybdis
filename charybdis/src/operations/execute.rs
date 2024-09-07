use scylla::serialize::row::SerializeRow;
use scylla::{CachingSession, QueryResult};

use crate::errors::CharybdisError;

pub async fn execute(
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
