use std::error::Error;
use std::fmt;

use colored::Colorize;
use scylla::_macro_internal::TypeCheckError;
use scylla::deserialize::DeserializationError;
use scylla::frame::value::SerializeValuesError;
use scylla::transport::errors::QueryError;
use scylla::transport::iterator::NextRowError;
use scylla::transport::query_result::{
    FirstRowError, IntoRowsResultError, MaybeFirstRowError, RowsError, SingleRowError,
};

#[derive(Debug)]
pub enum CharybdisError {
    // scylla
    QueryError(&'static str, QueryError),
    IntoRowsResultError(&'static str, IntoRowsResultError),
    BatchError(&'static str, QueryError),
    SingleRowError(&'static str, SingleRowError),
    RowsError(&'static str, RowsError),
    SerializeValuesError(&'static str, SerializeValuesError),
    FirstRowError(&'static str, FirstRowError),
    MaybeFirstRowError(&'static str, MaybeFirstRowError),
    DeserializationError(&'static str, DeserializationError),
    NextRowError(&'static str, NextRowError),
    TypeCheckError(&'static str, TypeCheckError),
    NotFoundError(&'static str),
    JsonError(serde_json::Error),
}

impl fmt::Display for CharybdisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // scylla errors
            CharybdisError::QueryError(query, e) => write!(f, "Query: {}\nQueryError: {}", query.bright_purple(), e),
            CharybdisError::IntoRowsResultError(query, e) => {
                write!(f, "Query: {}\nIntoRowsResultError: {}", query.bright_purple(), e)
            }
            CharybdisError::BatchError(query, e) => write!(f, "Model: {}\nBatchError: {}", query.bright_purple(), e),
            CharybdisError::SingleRowError(query, e) => write!(
                f,
                "Query: {}\nSingleRowError: {:?}. Did you forget to provide complete primary key?",
                query.bright_purple(),
                e
            ),
            CharybdisError::RowsError(query, e) => {
                write!(f, "Query: {}\nRowsError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::FirstRowError(query, e) => {
                write!(f, "Query: {}\nFirstRowError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::MaybeFirstRowError(query, e) => {
                write!(f, "Query: {}\nMaybeFirstRowError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::DeserializationError(query, e) => {
                write!(f, "Query: {}\nDeserializationError: {:?}", query.bright_purple(), e)
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
            CharybdisError::TypeCheckError(query, e) => {
                write!(f, "Query: {}\nTypeCheckError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::JsonError(e) => write!(f, "JsonError: {:?}", e),
        }
    }
}

impl Error for CharybdisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CharybdisError::QueryError(_, e) => Some(e),
            CharybdisError::IntoRowsResultError(_, e) => Some(e),
            CharybdisError::SingleRowError(_, e) => Some(e),
            CharybdisError::RowsError(_, e) => Some(e),
            CharybdisError::FirstRowError(_, e) => Some(e),
            CharybdisError::MaybeFirstRowError(_, e) => Some(e),
            CharybdisError::DeserializationError(_, e) => Some(e),
            CharybdisError::NextRowError(_, e) => Some(e),
            CharybdisError::SerializeValuesError(_, e) => Some(e),
            CharybdisError::JsonError(e) => Some(e),
            _ => None,
        }
    }
}
