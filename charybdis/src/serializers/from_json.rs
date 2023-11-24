use crate::errors::CharybdisError;
use crate::model::BaseModel;
use serde::Deserialize;

pub trait FromJson<'a, T: BaseModel + Deserialize<'a>> {
    fn from_json(json: &'a str) -> Result<T, CharybdisError>;
}

impl<'a, T: BaseModel + Deserialize<'a>> FromJson<'a, T> for T {
    fn from_json(json: &'a str) -> Result<T, CharybdisError> {
        serde_json::from_str(json).map_err(|e| CharybdisError::JsonError(e))
    }
}
