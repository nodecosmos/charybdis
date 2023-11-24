use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LocalIndexTarget {
    pub pk: Vec<String>,
    pub ck: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Target {
    GlobalSecondaryIndex(String),
    LocalIndexTarget(LocalIndexTarget),
}

// cql returns {'target': '{"pk":["node_id"],"ck":["id"]}'} for a local secondary index
// and {'target': 'node_id'} for a global secondary index
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexTarget {
    pub target: Target,
}

impl FromCqlVal<CqlValue> for IndexTarget {
    fn from_cql(value: CqlValue) -> Result<Self, FromCqlValError> {
        match value {
            CqlValue::Map(map) => {
                let target_value = map[0].1.clone();
                let target_val_string = target_value.into_string().unwrap();

                if target_val_string.starts_with('{') {
                    let parsed: LocalIndexTarget = serde_json::from_str(&target_val_string).unwrap();

                    return Ok(IndexTarget {
                        target: Target::LocalIndexTarget(parsed),
                    });
                }

                return Ok(IndexTarget {
                    target: Target::GlobalSecondaryIndex(target_val_string),
                });
            }
            _ => Err(FromCqlValError::BadVal),
        }
    }
}
