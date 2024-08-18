use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LocalIndexStructure {
    pub pk: Vec<String>,
    pub ck: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum IndexTarget {
    GlobalSecondaryIndex(String),
    LocalSecondaryIndex(String),
}

// cql returns {'target': '{"pk":["node_id"],"ck":["id"]}'} for a local secondary index
// and {'target': 'node_id'} for a global secondary index
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecondaryIndex {
    pub target: IndexTarget,
}

impl FromCqlVal<CqlValue> for SecondaryIndex {
    fn from_cql(value: CqlValue) -> Result<Self, FromCqlValError> {
        match value {
            CqlValue::Map(map) => {
                let target_value = map[0].1.clone();
                let target_val_string = target_value.into_string().unwrap();

                if target_val_string.starts_with('{') {
                    let parsed: LocalIndexStructure = serde_json::from_str(&target_val_string).unwrap();
                    let idx = parsed.ck.first().unwrap();

                    return Ok(SecondaryIndex {
                        target: IndexTarget::LocalSecondaryIndex(idx.to_string()),
                    });
                }

                Ok(SecondaryIndex {
                    target: IndexTarget::GlobalSecondaryIndex(target_val_string),
                })
            }
            _ => Err(FromCqlValError::BadVal),
        }
    }
}
