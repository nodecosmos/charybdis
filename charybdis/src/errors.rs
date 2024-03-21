use crate::FromRowError;
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
    QueryError(String, QueryError),
    RowsExpectedError(String, RowsExpectedError),
    SingleRowTypedError(String, SingleRowTypedError),
    SerializeValuesError(String, SerializeValuesError),
    FirstRowTypedError(String, FirstRowTypedError),
    MaybeFirstRowTypedError(String, MaybeFirstRowTypedError),
    FromRowError(String, FromRowError),
    NextRowError(String, NextRowError),
    NotFoundError(String),
    JsonError(serde_json::Error),
}

impl fmt::Display for CharybdisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // scylla errors
            CharybdisError::QueryError(query, e) => write!(f, "Query: {}\nQueryError: {}", query, e),
            CharybdisError::RowsExpectedError(query, e) => {
                write!(f, "Query: {}\nRowsExpectedError: {:?}", query, e)
            }
            CharybdisError::SingleRowTypedError(query, e) => write!(
                f,
                "Query: {}\nSingleRowTypedError: {:?}. Did you forget to provide complete primary key?",
                query, e
            ),
            CharybdisError::FirstRowTypedError(query, e) => {
                write!(f, "Query: {}\nFirstRowTypedError: {:?}", query, e)
            }
            CharybdisError::MaybeFirstRowTypedError(query, e) => {
                write!(f, "Query: {}\nMaybeFirstRowTypedError: {:?}", query, e)
            }
            CharybdisError::FromRowError(query, e) => {
                write!(f, "Query: {}\nFromRowError: {:?}", query, e)
            }

            CharybdisError::SerializeValuesError(query, e) => {
                write!(f, "Query: {}\nSerializeValuesError: {:?}", query, e)
            }
            CharybdisError::NotFoundError(query) => {
                write!(f, "Records not found for query: {}", query)
            }
            CharybdisError::NextRowError(query, e) => write!(f, "Query: {}\nNextRowError: {:?}", query, e),
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
