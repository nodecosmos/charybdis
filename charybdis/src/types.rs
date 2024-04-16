use crate::{FromCqlVal, SerializeCql};
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, Utc};
use scylla::_macro_internal::{CellWriter, ColumnType, FromCqlValError, SerializationError, WrittenCellProof};
use scylla::frame::response::result::CqlValue;
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
pub type Duration = String; // TODO: Duration as native type in scylla does not have serde support
pub type Float = f32;
pub type Inet = IpAddr;
pub type Int = i32;
pub type SmallInt = i16;
pub type Text = String;
pub type Time = chrono::DateTime<Utc>;
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

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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
