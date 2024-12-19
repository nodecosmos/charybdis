use charybdis::types::{Boolean, Text, Timestamp, Uuid};
use charybdis_macros::{charybdis_model, charybdis_udt_model};

#[derive(Default, Clone)]
#[charybdis_udt_model(type_name = address)]
pub struct Address {
    pub street: Text,
    pub city: Text,
    pub state: Text,
    pub zip: Text,
    pub country: Text,
}

#[charybdis_model(
    table_name = users,
    partition_keys = [id],
    clustering_keys = [],
    global_secondary_indexes = [username, email],
)]
#[derive(Default)]
pub struct User {
    pub id: Uuid,
    pub username: Text,
    pub email: Text,
    pub password: Text,
    pub first_name: Text,
    pub last_name: Text,
    pub bio: Option<Text>,
    pub address: Option<Address>,
    pub is_confirmed: Boolean,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
