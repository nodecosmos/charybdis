use crate::errors::CharybdisError;
use crate::SerializeRow;
use scylla::query::Query;
use scylla::{CachingSession, QueryResult};

pub async fn execute(
    session: &CachingSession,
    query: impl Into<Query>,
    values: impl SerializeRow,
) -> Result<QueryResult, CharybdisError> {
    let res = session.execute(query, values).await?;
    Ok(res)
}
