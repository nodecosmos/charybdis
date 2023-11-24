use crate::errors::CharybdisError;
use crate::model::BaseModel;
use serde::Serialize;

pub trait ToJson<T: BaseModel + Serialize> {
    fn from_json(json: &str) -> Result<String, CharybdisError>;
}

impl<T: BaseModel + Serialize> ToJson<T> for T {
    fn from_json(json: &str) -> Result<String, CharybdisError> {
        serde_json::to_string(json).map_err(|e| CharybdisError::JsonError(e))
    }
}
