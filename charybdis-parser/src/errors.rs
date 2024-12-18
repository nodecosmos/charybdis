use std::error::Error;
use std::fmt;

use scylla::deserialize::DeserializationError;
use scylla::transport::errors::QueryError;
use scylla::transport::query_result::{IntoRowsResultError, RowsError};

#[derive(Debug)]
pub enum DbSchemaParserError {
    // scylla
    QueryError(QueryError),
    IntoRowsResultError(IntoRowsResultError),
    RowsError(RowsError),
    DeserializationError(DeserializationError),
    TypeError(String),
}

impl Error for DbSchemaParserError {}

impl From<QueryError> for DbSchemaParserError {
    fn from(e: QueryError) -> Self {
        DbSchemaParserError::QueryError(e)
    }
}

impl From<IntoRowsResultError> for DbSchemaParserError {
    fn from(e: IntoRowsResultError) -> Self {
        DbSchemaParserError::IntoRowsResultError(e)
    }
}

impl From<DeserializationError> for DbSchemaParserError {
    fn from(e: DeserializationError) -> Self {
        DbSchemaParserError::DeserializationError(e)
    }
}

impl From<RowsError> for DbSchemaParserError {
    fn from(e: RowsError) -> Self {
        DbSchemaParserError::RowsError(e)
    }
}

impl fmt::Display for DbSchemaParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // scylla errors
            DbSchemaParserError::QueryError(e) => write!(f, "QueryError: {}", e),
            DbSchemaParserError::IntoRowsResultError(e) => {
                write!(f, "IntoRowsResultError: {:?}", e)
            }
            DbSchemaParserError::RowsError(e) => {
                write!(f, "RowsError: {:?}", e)
            }
            DbSchemaParserError::DeserializationError(e) => {
                write!(f, "DeserializationError: {:?}", e)
            }
            DbSchemaParserError::TypeError(e) => {
                write!(f, "TypeError: {:?}", e)
            }
        }
    }
}
