use crate::errors::CharybdisError;
use crate::model::BaseModel;
use serde::Serialize;

pub trait ToJson<T: BaseModel + Serialize> {
    fn to_json(&self) -> Result<String, CharybdisError>;
}

impl<T: BaseModel + Serialize> ToJson<T> for T {
    fn to_json(&self) -> Result<String, CharybdisError> {
        serde_json::to_string(&self).map_err(CharybdisError::JsonError)
    }
}
