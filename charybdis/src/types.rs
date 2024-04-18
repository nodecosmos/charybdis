use crate::{FromCqlVal, SerializeCql};
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, Utc};
use scylla::_macro_internal::{CellWriter, ColumnType, FromCqlValError, SerializationError, WrittenCellProof};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::CqlDuration;
pub use scylla::macros::{FromRow, FromUserType, IntoUserType, ValueList};
use scylla::serialize::value::{BuiltinTypeCheckError, BuiltinTypeCheckErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

pub type Ascii = String;
pub type BigInt = i64;
pub type Boolean = bool;
pub type Blob = Vec<u8>;
pub type Date = NaiveDate;
pub type Decimal = BigDecimal;
pub type Double = f64;
pub type Float = f32;
pub type Inet = IpAddr;
pub type Int = i32;
pub type SmallInt = i16;
pub type Text = String;
pub type Time = chrono::NaiveTime;
pub type Timestamp = chrono::DateTime<Utc>;
pub type Timeuuid = Uuid;
pub type TinyInt = i8;
pub type Uuid = uuid::Uuid;
pub type Varchar = String;
pub type Varint = BigInt;
// collections
pub type Map<K, V> = HashMap<K, V>;
pub type List<T> = Vec<T>;
pub type Set<T> = HashSet<T>;
pub type Tuple<T1, T2> = (T1, T2);
// TODO: Tuple with more than 2 elements

pub type Frozen<T> = T;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Counter(pub i64);

impl FromCqlVal<CqlValue> for Counter {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let counter_value = scylla::frame::value::Counter::from_cql(cql_val)?;
        Ok(Counter(counter_value.0))
    }
}

impl SerializeCql for Counter {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        if typ != &ColumnType::Counter {
            return Err(SerializationError::new(BuiltinTypeCheckError {
                rust_name: std::any::type_name::<Counter>(),
                got: typ.clone(),
                kind: BuiltinTypeCheckErrorKind::MismatchedType {
                    expected: &[ColumnType::Counter],
                },
            }));
        }

        let proof = writer.set_value(self.0.to_be_bytes().as_slice()).unwrap();

        Ok(proof)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Duration(pub CqlDuration);

impl Duration {
    pub fn new(months: i32, days: i32, nanoseconds: i64) -> Self {
        Duration(CqlDuration {
            months,
            days,
            nanoseconds,
        })
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration(CqlDuration {
            months: 0,
            days: 0,
            nanoseconds: 0,
        })
    }
}
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Helper {
            months: i32,
            days: i32,
            nanoseconds: i64,
        }

        let helper = Helper {
            months: self.0.months,
            days: self.0.days,
            nanoseconds: self.0.nanoseconds,
        };

        helper.serialize(serializer)
    }
}

// Implementing Deserialize for Duration
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            months: i32,
            days: i32,
            nanoseconds: i64,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(Duration(CqlDuration {
            months: helper.months,
            days: helper.days,
            nanoseconds: helper.nanoseconds,
        }))
    }
}

impl FromCqlVal<CqlValue> for Duration {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let duration = CqlDuration::from_cql(cql_val)?;
        Ok(Duration(duration))
    }
}

impl SerializeCql for Duration {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}
