use charybdis::scylla::serialize::writers::CellWriter;
use charybdis::scylla::SerializeValue;
use charybdis::types::Text;
use scylla::deserialize::value::DeserializeValue;
use scylla::deserialize::{DeserializationError, FrameSlice, TypeCheckError};
use scylla::frame::response::result::ColumnType;
use scylla::serialize::writers::WrittenCellProof;
use scylla::serialize::SerializationError;

#[derive(Debug, Default, Clone, PartialEq, strum::FromRepr)]
#[repr(i8)]
pub enum AddressTypeCustomField {
    #[default]
    HomeAddress = 0,
    WorkAddress = 1,
}

#[derive(Debug)]
struct AddressTypeCustomDeserializeErr(i8);
impl std::fmt::Display for AddressTypeCustomDeserializeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddressTypeCustomDeserializeErr({})", self.0)
    }
}
impl std::error::Error for AddressTypeCustomDeserializeErr {}

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for AddressTypeCustomField {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        <i8 as DeserializeValue<'frame, 'metadata>>::type_check(typ)
    }

    fn deserialize(
        typ: &'metadata ColumnType<'metadata>,
        v: Option<FrameSlice<'frame>>,
    ) -> Result<Self, DeserializationError> {
        let si8 = <i8 as DeserializeValue<'frame, 'metadata>>::deserialize(typ, v)?;
        let s = Self::from_repr(si8);
        s.ok_or_else(|| DeserializationError::new(AddressTypeCustomDeserializeErr(si8)))
    }
}

impl SerializeValue for AddressTypeCustomField {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        let disc = self.clone() as i8;

        let v = <i8 as SerializeValue>::serialize(&disc, typ, writer)?;
        Ok(v)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserExtraDataCustomField {
    pub user_tags: Vec<(String, String)>,
}

impl Default for UserExtraDataCustomField {
    fn default() -> Self {
        Self {
            user_tags: vec![("some_key".to_string(), "some_value".to_string())],
        }
    }
}

#[derive(Debug)]
struct UserExtraDataDeserializeErr(String);
impl std::fmt::Display for UserExtraDataDeserializeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserExtraDataDeserializeErr({})", self.0)
    }
}
impl std::error::Error for UserExtraDataDeserializeErr {}

impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for UserExtraDataCustomField {
    fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
        <Text as DeserializeValue<'frame, 'metadata>>::type_check(typ)
    }

    fn deserialize(
        typ: &'metadata ColumnType<'metadata>,
        v: Option<FrameSlice<'frame>>,
    ) -> Result<Self, DeserializationError> {
        let si8 = <Text as DeserializeValue<'frame, 'metadata>>::deserialize(typ, v)?;
        serde_json::from_str::<UserExtraDataCustomField>(&si8)
            .map_err(|_e| DeserializationError::new(UserExtraDataDeserializeErr(si8)))
    }
}

impl SerializeValue for UserExtraDataCustomField {
    fn serialize<'b>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'b>,
    ) -> Result<WrittenCellProof<'b>, SerializationError> {
        let disc = serde_json::to_string(&self).map_err(|_e| SerializationError::new(_e))?;

        let v = <Text as SerializeValue>::serialize(&disc, typ, writer)?;
        Ok(v)
    }
}
