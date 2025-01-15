use charybdis::macros::charybdis_udt_model;
use charybdis::types::Text;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[charybdis_udt_model(type_name = address)]
pub struct Address {
    pub street: Text,
    pub city: Text,
    pub state: Text,
    pub zip: Text,
    pub country: Text,
}
