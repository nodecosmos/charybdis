use crate::FromRowError;
use colored::Colorize;
use scylla::frame::value::SerializeValuesError;
use scylla::transport::errors::QueryError;
use scylla::transport::iterator::NextRowError;
use scylla::transport::query_result::{
    FirstRowTypedError, MaybeFirstRowTypedError, RowsExpectedError, SingleRowTypedError,
};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CharybdisError {
    // scylla
    QueryError(&'static str, QueryError),
    BatchError(&'static str, QueryError),
    RowsExpectedError(&'static str, RowsExpectedError),
    SingleRowTypedError(&'static str, SingleRowTypedError),
    SerializeValuesError(&'static str, SerializeValuesError),
    FirstRowTypedError(&'static str, FirstRowTypedError),
    MaybeFirstRowTypedError(&'static str, MaybeFirstRowTypedError),
    FromRowError(&'static str, FromRowError),
    NextRowError(&'static str, NextRowError),
    NotFoundError(&'static str),
    JsonError(serde_json::Error),
}

impl fmt::Display for CharybdisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // scylla errors
            CharybdisError::QueryError(query, e) => write!(f, "Query: {}\nQueryError: {}", query.bright_purple(), e),
            CharybdisError::BatchError(query, e) => write!(f, "Model: {}\nBatchError: {}", query.bright_purple(), e),
            CharybdisError::RowsExpectedError(query, e) => {
                write!(f, "Query: {}\nRowsExpectedError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::SingleRowTypedError(query, e) => write!(
                f,
                "Query: {}\nSingleRowTypedError: {:?}. Did you forget to provide complete primary key?",
                query.bright_purple(),
                e
            ),
            CharybdisError::FirstRowTypedError(query, e) => {
                write!(f, "Query: {}\nFirstRowTypedError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::MaybeFirstRowTypedError(query, e) => {
                write!(f, "Query: {}\nMaybeFirstRowTypedError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::FromRowError(query, e) => {
                write!(f, "Query: {}\nFromRowError: {:?}", query.bright_purple(), e)
            }

            CharybdisError::SerializeValuesError(query, e) => {
                write!(f, "Query: {}\nSerializeValuesError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::NotFoundError(query) => {
                write!(f, "Records not found for query: {}", query.bright_purple())
            }
            CharybdisError::NextRowError(query, e) => {
                write!(f, "Query: {}\nNextRowError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::JsonError(e) => write!(f, "JsonError: {:?}", e),
        }
    }
}

impl Error for CharybdisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CharybdisError::QueryError(_, e) => Some(e),
            CharybdisError::RowsExpectedError(_, e) => Some(e),
            CharybdisError::SingleRowTypedError(_, e) => Some(e),
            CharybdisError::FirstRowTypedError(_, e) => Some(e),
            CharybdisError::MaybeFirstRowTypedError(_, e) => Some(e),
            CharybdisError::FromRowError(_, e) => Some(e),
            CharybdisError::NextRowError(_, e) => Some(e),
            CharybdisError::SerializeValuesError(_, e) => Some(e),
            CharybdisError::JsonError(e) => Some(e),
            _ => None,
        }
    }
}
