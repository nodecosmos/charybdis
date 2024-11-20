use crate::errors::DbSchemaParserError;
use scylla::_macro_internal::ColumnType;
use scylla::deserialize::{DeserializationError, DeserializeValue, FrameSlice, TypeCheckError};
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecondaryIndex {
    pub target: IndexTarget,
}

// cql returns {'target': '{"pk":["node_id"],"ck":["id"]}'} for a local secondary index
// and {'target': 'node_id'} for a global secondary index
impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for SecondaryIndex {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        if let ColumnType::Map(_, _) = typ {
            Ok(())
        } else {
            Err(TypeCheckError::new(DbSchemaParserError::TypeError(
                "Expected a map".to_string(),
            )))
        }
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
                        // global secondary index

                        Ok(SecondaryIndex {
                            target: IndexTarget::GlobalSecondaryIndex(target),
                        })
                    } else if let CqlValue::Map(map) = target {
                        // local secondary index

                        let mut ck = Vec::new();
                        for (key, value) in map {
                            if let CqlValue::Text(key_str) = key {
                                // parse only 'ck' key
                                if key_str == "ck" {
                                    if let CqlValue::List(ck_list) = value {
                                        for ck_val in ck_list {
                                            if let CqlValue::Text(ck_str) = ck_val {
                                                ck.push(ck_str);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(SecondaryIndex {
                            target: IndexTarget::LocalSecondaryIndex(ck.first().expect("Expected a string").clone()),
                        })
                    } else {
                        Err(DeserializationError::new(DbSchemaParserError::TypeError(
                            "Expected a map for secondary index".to_string(),
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
