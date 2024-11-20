use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, Utc};
use scylla::_macro_internal::{CellWriter, ColumnType, SerializationError, WrittenCellProof};
use scylla::deserialize::{DeserializationError, DeserializeValue, FrameSlice, TypeCheckError};
use scylla::frame::value::{Counter as CqlCounter, CqlDuration, CqlTimeuuid};
use scylla::serialize::value::{BuiltinTypeCheckError, SerializeValue};
use serde::{Deserialize, Serialize};

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

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for Counter {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        CqlCounter::type_check(typ)
    }

    fn deserialize(
        typ: &'metadata ColumnType<'metadata>,
        v: Option<FrameSlice<'frame>>,
    ) -> Result<Self, DeserializationError> {
        let counter = CqlCounter::deserialize(typ, v)?;
        Ok(Counter(counter.0))
    }
}

impl SerializeValue for Counter {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        if typ != &ColumnType::Counter {
            return Err(SerializationError::new(BuiltinTypeCheckError {
                rust_name: std::any::type_name::<Counter>(),
                got: typ.clone().into_owned(),
                kind: scylla::serialize::value::BuiltinTypeCheckErrorKind::MismatchedType {
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

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for Duration {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        CqlDuration::type_check(typ)
    }

    fn deserialize(
        typ: &'metadata ColumnType<'metadata>,
        v: Option<FrameSlice<'frame>>,
    ) -> Result<Self, DeserializationError> {
        let duration = CqlDuration::deserialize(typ, v)?;

        Ok(Duration(duration))
    }
}

impl SerializeValue for Duration {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}

/// Bellow is copy paste from scylla's CqlTimeuuid, with serde support added.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, Eq)]
pub struct Timeuuid(Uuid);

/// [`Uuid`] delegate methods
impl Timeuuid {
    pub fn now_v1(node_id: &[u8; 6]) -> Self {
        Self(Uuid::now_v1(node_id))
    }

    pub fn new_v1(ts: uuid::Timestamp, node_id: &[u8; 6]) -> Self {
        Self(Uuid::new_v1(ts, node_id))
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }

    pub fn as_u128(&self) -> u128 {
        self.0.as_u128()
    }

    pub fn as_fields(&self) -> (u32, u16, u16, &[u8; 8]) {
        self.0.as_fields()
    }

    pub fn as_u64_pair(&self) -> (u64, u64) {
        self.0.as_u64_pair()
    }

    pub fn from_slice(b: &[u8]) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::from_slice(b)?))
    }

    pub fn from_slice_le(b: &[u8]) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::from_slice_le(b)?))
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(Uuid::from_bytes(bytes))
    }

    pub fn from_bytes_le(bytes: [u8; 16]) -> Self {
        Self(Uuid::from_bytes_le(bytes))
    }

    pub fn from_fields(d1: u32, d2: u16, d3: u16, d4: &[u8; 8]) -> Self {
        Self(Uuid::from_fields(d1, d2, d3, d4))
    }

    pub fn from_fields_le(d1: u32, d2: u16, d3: u16, d4: &[u8; 8]) -> Self {
        Self(Uuid::from_fields_le(d1, d2, d3, d4))
    }

    pub fn from_u128(v: u128) -> Self {
        Self(Uuid::from_u128(v))
    }

    pub fn from_u128_le(v: u128) -> Self {
        Self(Uuid::from_u128_le(v))
    }

    pub fn from_u64_pair(high_bits: u64, low_bits: u64) -> Self {
        Self(Uuid::from_u64_pair(high_bits, low_bits))
    }
}

impl Timeuuid {
    /// Read 8 most significant bytes of timeuuid from serialized bytes
    fn msb(&self) -> u64 {
        // Scylla and Cassandra use a standard UUID memory layout for MSB:
        // 4 bytes    2 bytes    2 bytes
        // time_low - time_mid - time_hi_and_version
        let bytes = self.0.as_bytes();
        ((bytes[6] & 0x0F) as u64) << 56
            | (bytes[7] as u64) << 48
            | (bytes[4] as u64) << 40
            | (bytes[5] as u64) << 32
            | (bytes[0] as u64) << 24
            | (bytes[1] as u64) << 16
            | (bytes[2] as u64) << 8
            | (bytes[3] as u64)
    }

    fn lsb(&self) -> u64 {
        let bytes = self.0.as_bytes();
        (bytes[8] as u64) << 56
            | (bytes[9] as u64) << 48
            | (bytes[10] as u64) << 40
            | (bytes[11] as u64) << 32
            | (bytes[12] as u64) << 24
            | (bytes[13] as u64) << 16
            | (bytes[14] as u64) << 8
            | (bytes[15] as u64)
    }

    fn lsb_signed(&self) -> u64 {
        self.lsb() ^ 0x8080808080808080
    }
}

impl std::str::FromStr for Timeuuid {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::from_str(s)?))
    }
}

impl std::fmt::Display for Timeuuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Uuid> for Timeuuid {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl From<Timeuuid> for Uuid {
    fn from(value: Timeuuid) -> Self {
        value.0
    }
}

impl From<Uuid> for Timeuuid {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

/// Compare two values of timeuuid type.
///
/// Cassandra legacy requires:
/// - converting 8 most significant bytes to date, which is then compared.
/// - masking off UUID version from the 8 ms-bytes during compare, to
///   treat possible non-version-1 UUID the same way as UUID.
/// - using signed compare for least significant bits.
impl Ord for Timeuuid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let mut res = self.msb().cmp(&other.msb());
        if let std::cmp::Ordering::Equal = res {
            res = self.lsb_signed().cmp(&other.lsb_signed());
        }
        res
    }
}

impl PartialOrd for Timeuuid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timeuuid {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl std::hash::Hash for Timeuuid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lsb_signed().hash(state);
        self.msb().hash(state);
    }
}

/// Driver traits
impl From<CqlTimeuuid> for Timeuuid {
    fn from(cql_timeuuid: CqlTimeuuid) -> Self {
        // we can not return CqlTimeuuid.0 as it's private
        let u128 = cql_timeuuid.as_u128();
        let uuid = Uuid::from_u128(u128);

        Timeuuid(uuid)
    }
}

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for Timeuuid {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        CqlTimeuuid::type_check(typ)
    }

    fn deserialize(
        typ: &'metadata ColumnType<'metadata>,
        v: Option<FrameSlice<'frame>>,
    ) -> Result<Self, DeserializationError> {
        let uuid = CqlTimeuuid::deserialize(typ, v)?;
        Ok(Timeuuid::from(uuid))
    }
}

impl SerializeValue for Timeuuid {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        if typ != &ColumnType::Timeuuid {
            return Err(SerializationError::new(BuiltinTypeCheckError {
                rust_name: std::any::type_name::<Timeuuid>(),
                got: typ.clone().into_owned(),
                kind: scylla::serialize::value::BuiltinTypeCheckErrorKind::MismatchedType {
                    expected: &[ColumnType::Timeuuid],
                },
            }));
        }

        let proof = writer.set_value(self.as_bytes().as_ref()).unwrap();

        Ok(proof)
    }
}
