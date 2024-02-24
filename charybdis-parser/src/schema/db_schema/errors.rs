use scylla::cql_to_rust::FromRowError;
use scylla::transport::errors::QueryError;
use std::fmt;

#[derive(Debug)]
pub enum DbSchemaParserError {
    // scylla
    QueryError(QueryError),
    FromRowError(FromRowError),
}

impl From<QueryError> for DbSchemaParserError {
    fn from(e: QueryError) -> Self {
        DbSchemaParserError::QueryError(e)
    }
}

impl From<FromRowError> for DbSchemaParserError {
    fn from(e: FromRowError) -> Self {
        DbSchemaParserError::FromRowError(e)
    }
}

impl fmt::Display for DbSchemaParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // scylla errors
            DbSchemaParserError::QueryError(e) => write!(f, "QueryError: {}", e),
            DbSchemaParserError::FromRowError(e) => {
                write!(f, "FromRowError: {:?}", e)
            }
        }
    }
}
