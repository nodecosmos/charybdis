use std::error::Error;
use std::fmt;

use scylla::deserialize::DeserializationError;
use scylla::errors::{ExecutionError, IntoRowsResultError, RowsError};

#[derive(Debug)]
pub enum DbSchemaParserError {
    // scylla
    ExecutionError(ExecutionError),
    IntoRowsResultError(IntoRowsResultError),
    RowsError(RowsError),
    DeserializationError(DeserializationError),
    TypeError(String),
}

impl Error for DbSchemaParserError {}

impl From<ExecutionError> for DbSchemaParserError {
    fn from(e: ExecutionError) -> Self {
        DbSchemaParserError::ExecutionError(e)
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
            DbSchemaParserError::ExecutionError(e) => write!(f, "ExecutionError: {}", e),
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
