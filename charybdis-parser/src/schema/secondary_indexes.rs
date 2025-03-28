use crate::errors::DbSchemaParserError;
use scylla::_macro_internal::{ColumnType, DeserializeValue};
use scylla::cluster::metadata::CollectionType;
use scylla::deserialize::{DeserializationError, FrameSlice, TypeCheckError};
use scylla::value::CqlValue;
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecondaryIndex {
    pub target: IndexTarget,
}

// cql returns {'target': '{"pk":["node_id"],"ck":["id"]}'} for a local secondary index
// and {'target': 'node_id'} for a global secondary index
impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for SecondaryIndex {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        if let ColumnType::Collection { frozen: _frozen, typ } = typ {
            if let CollectionType::Map(_, _) = typ {
                return Ok(());
            }
        }

        Err(TypeCheckError::new(DbSchemaParserError::TypeError(
            "Expected a map".to_string(),
        )))
    }

    fn deserialize(
        typ: &'metadata ColumnType<'metadata>,
        v: Option<FrameSlice<'frame>>,
    ) -> Result<Self, DeserializationError> {
        match v {
            Some(slice) => {
                let value: CqlValue = CqlValue::deserialize(typ, Some(slice))?;
                if let CqlValue::Map(map) = value {
                    let target = map[0].1.clone();

                    if let CqlValue::Text(target) = target {
                        // local secondary index
                        if target.starts_with('{') {
                            let parsed: LocalIndexStructure =
                                serde_json::from_str(&target).expect("Failed to parse local index structure");
                            let idx = parsed.ck.first().expect("Expected CK key");

                            return Ok(SecondaryIndex {
                                target: IndexTarget::LocalSecondaryIndex(idx.to_string()),
                            });
                        }

                        // global secondary index
                        Ok(SecondaryIndex {
                            target: IndexTarget::GlobalSecondaryIndex(target),
                        })
                    } else {
                        Err(DeserializationError::new(DbSchemaParserError::TypeError(
                            "Expected a string".to_string(),
                        )))
                    }
                } else {
                    Err(DeserializationError::new(DbSchemaParserError::TypeError(
                        "Expected a map".to_string(),
                    )))
                }
            }
            None => Err(DeserializationError::new(DbSchemaParserError::TypeError(
                "No value to deserialize".to_string(),
            ))),
        }
    }
}
