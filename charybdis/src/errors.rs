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
    QueryError(QueryError),
    RowsExpectedError(RowsExpectedError, String),
    SingleRowTypedError(SingleRowTypedError, String),
    SerializeValuesError(SerializeValuesError, String),
    FirstRowTypedError(FirstRowTypedError, String),
    MaybeFirstRowTypedError(MaybeFirstRowTypedError, String),
    FromRowError(FromRowError, String),
    NextRowError(NextRowError),
    SchemaError(String),
    // charybdis
    NotFoundError(String),
    // serde
    JsonError(serde_json::Error),
}

impl fmt::Display for CharybdisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // scylla errors
            CharybdisError::QueryError(e) => write!(f, "QueryError: {}", e),
            CharybdisError::RowsExpectedError(e, model_name) => {
                write!(f, "RowsExpectedError: {:?} \nin Mode: {}", e, model_name)
            }
            CharybdisError::SingleRowTypedError(e, model_name) => write!(
                f,
                "SingleRowTypedError: {:?} \n in Model: {}. Did you forget to provide complete primary key?",
                e, model_name
            ),
            CharybdisError::FirstRowTypedError(e, model) => {
                write!(f, "FirstRowTypedError: {:?} \nin Model: {}", e, model)
            }
            CharybdisError::MaybeFirstRowTypedError(e, model) => {
                write!(f, "FirstRowTypedError: {:?} \nin Model: {}", e, model)
            }
            CharybdisError::FromRowError(e, model) => {
                write!(f, "FromRowError: {:?} \nin Model: {}", e, model)
            }
            CharybdisError::SerializeValuesError(e, model) => {
                write!(f, "SerializeValuesError: {}\n{}", e, model)
            }
            CharybdisError::NextRowError(e) => write!(f, "NextRowError: {}", e),

            // charybdis
            CharybdisError::NotFoundError(e) => write!(f, "Records not found for query: {}", e),

            // serde
            CharybdisError::JsonError(e) => write!(f, "JsonError: {}", e),

            CharybdisError::SchemaError(e) => write!(f, "SchemaError: {}", e),
        }
    }
}

impl Error for CharybdisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CharybdisError::QueryError(e) => Some(e),
            CharybdisError::RowsExpectedError(e, _) => Some(e),
            CharybdisError::NotFoundError(_) => None,
            CharybdisError::SingleRowTypedError(e, _) => Some(e),
            CharybdisError::FirstRowTypedError(e, _) => Some(e),
            CharybdisError::MaybeFirstRowTypedError(e, _) => Some(e),
            CharybdisError::FromRowError(e, _) => Some(e),
            CharybdisError::NextRowError(e) => Some(e),
            CharybdisError::SerializeValuesError(e, _) => Some(e),
            CharybdisError::JsonError(e) => Some(e),
            CharybdisError::SchemaError(_) => None,
        }
    }
}

impl From<QueryError> for CharybdisError {
    fn from(e: QueryError) -> Self {
        CharybdisError::QueryError(e)
    }
}

impl From<RowsExpectedError> for CharybdisError {
    fn from(e: RowsExpectedError) -> Self {
        CharybdisError::RowsExpectedError(e, "unknown".to_string())
    }
}

impl From<SingleRowTypedError> for CharybdisError {
    fn from(e: SingleRowTypedError) -> Self {
        CharybdisError::SingleRowTypedError(e, "unknown".to_string())
    }
}

impl From<FirstRowTypedError> for CharybdisError {
    fn from(e: FirstRowTypedError) -> Self {
        match e {
            FirstRowTypedError::RowsEmpty => CharybdisError::NotFoundError(e.to_string()),
            _ => CharybdisError::FirstRowTypedError(e, "unknown".to_string()),
        }
    }
}

impl From<FromRowError> for CharybdisError {
    fn from(e: FromRowError) -> Self {
        CharybdisError::FromRowError(e, "unknown".to_string())
    }
}

impl From<SerializeValuesError> for CharybdisError {
    fn from(e: SerializeValuesError) -> Self {
        CharybdisError::SerializeValuesError(e, "unknown".to_string())
    }
}

impl From<NextRowError> for CharybdisError {
    fn from(e: NextRowError) -> Self {
        CharybdisError::NextRowError(e)
    }
}
