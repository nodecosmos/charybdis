use std::error::Error;
use std::fmt;

use colored::Colorize;
use scylla::_macro_internal::TypeCheckError;
use scylla::deserialize::DeserializationError;
use scylla::errors::{
    ExecutionError, FirstRowError, IntoRowsResultError, MaybeFirstRowError, NextRowError, PagerExecutionError,
    RowsError, SingleRowError,
};

#[derive(Debug)]
pub enum CharybdisError {
    // scylla
    ExecutionError(&'static str, ExecutionError),
    PagerExecutionError(&'static str, PagerExecutionError),
    IntoRowsResultError(&'static str, IntoRowsResultError),
    BatchError(&'static str, ExecutionError),
    SingleRowError(&'static str, SingleRowError),
    RowsError(&'static str, RowsError),
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
            CharybdisError::ExecutionError(query, e) => {
                write!(f, "Statement: {}\nExecutionError: {}", query.bright_purple(), e)
            }
            CharybdisError::PagerExecutionError(query, e) => {
                write!(f, "Statement: {}\nPagerExecutionError: {}", query.bright_purple(), e)
            }
            CharybdisError::IntoRowsResultError(query, e) => {
                write!(f, "Statement: {}\nIntoRowsResultError: {}", query.bright_purple(), e)
            }
            CharybdisError::BatchError(query, e) => write!(f, "Model: {}\nBatchError: {}", query.bright_purple(), e),
            CharybdisError::SingleRowError(query, e) => write!(
                f,
                "Statement: {}\nSingleRowError: {:?}. Did you forget to provide complete primary key?",
                query.bright_purple(),
                e
            ),
            CharybdisError::RowsError(query, e) => {
                write!(f, "Statement: {}\nRowsError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::FirstRowError(query, e) => {
                write!(f, "Statement: {}\nFirstRowError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::MaybeFirstRowError(query, e) => {
                write!(f, "Statement: {}\nMaybeFirstRowError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::DeserializationError(query, e) => {
                write!(f, "Statement: {}\nDeserializationError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::NotFoundError(query) => {
                write!(f, "Records not found for query: {}", query.bright_purple())
            }
            CharybdisError::NextRowError(query, e) => {
                write!(f, "Statement: {}\nNextRowError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::TypeCheckError(query, e) => {
                write!(f, "Statement: {}\nTypeCheckError: {:?}", query.bright_purple(), e)
            }
            CharybdisError::JsonError(e) => write!(f, "JsonError: {:?}", e),
        }
    }
}

impl Error for CharybdisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CharybdisError::ExecutionError(_, e) => Some(e),
            CharybdisError::PagerExecutionError(_, e) => Some(e),
            CharybdisError::IntoRowsResultError(_, e) => Some(e),
            CharybdisError::SingleRowError(_, e) => Some(e),
            CharybdisError::RowsError(_, e) => Some(e),
            CharybdisError::FirstRowError(_, e) => Some(e),
            CharybdisError::MaybeFirstRowError(_, e) => Some(e),
            CharybdisError::DeserializationError(_, e) => Some(e),
            CharybdisError::NextRowError(_, e) => Some(e),
            CharybdisError::JsonError(e) => Some(e),
            _ => None,
        }
    }
}
