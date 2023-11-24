use crate::FromCqlVal;
use chrono::{NaiveDate, Utc};
use scylla::_macro_internal::{FromCqlValError, ValueTooBig};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::Value;
pub use scylla::macros::{FromRow, FromUserType, IntoUserType, ValueList};
use scylla::BufMut;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

pub type Ascii = String;
pub type BigInt = i64;
pub type Boolean = bool;
pub type Blob = Vec<u8>;
pub type Date = NaiveDate;
pub type Decimal = f64;
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
pub type Set<T> = Vec<T>;
pub type Tuple = Vec<Option<CqlValue>>;

pub type Frozen<T> = T;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Counter(pub i64);

impl FromCqlVal<CqlValue> for Counter {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let counter_value = scylla::frame::value::Counter::from_cql(cql_val)?;
        Ok(Counter(counter_value.0))
    }
}

impl Value for Counter {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        buf.put_i32(8);
        buf.put_i64(self.0);
        Ok(())
    }
}
