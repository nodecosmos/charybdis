use crate::errors::AppError;
use crate::models::udts::Address;
use charybdis::callbacks::Callbacks;
use charybdis::macros::charybdis_model;
use charybdis::scylla::CachingSession;
use charybdis::types::{Text, Timestamp, Uuid};
use serde::{Deserialize, Serialize};

#[charybdis_model(
    table_name = users,
    partition_keys = [id],
    clustering_keys = [],
    global_secondary_indexes = [username],
)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    pub username: Text,
    pub email: Text,

    pub password: Text,

    pub first_name: Text,
    pub last_name: Text,
    pub bio: Option<Text>,
    pub address: Option<Address>,

    #[serde(default = "chrono::Utc::now")]
    pub created_at: Timestamp,

    #[serde(default = "chrono::Utc::now")]
    pub updated_at: Timestamp,
}

impl Callbacks for User {
    type Extension = ();
    type Error = AppError;

    async fn before_insert(
        &mut self,
        _session: &CachingSession,
        _extension: &Self::Extension,
    ) -> Result<(), Self::Error> {
        self.id = Uuid::new_v4();
        self.created_at = chrono::Utc::now();
        self.updated_at = chrono::Utc::now();

        Ok(())
    }
}

// use partial user to avoid rendering password
partial_user!(ShowUser, id, username, first_name, last_name, bio, address, created_at, updated_at);

#[cfg(test)]
mod tests {
    use crate::models::udts::Address;
    use crate::models::user::User;
    use charybdis::types::Uuid;

    impl User {
        pub fn homer() -> User {
            User {
                id: Uuid::nil(),
                username: "test".to_string(),
                password: "Marge".to_string(),
                first_name: "Homer".to_string(),
                email: "homer@simpson.com".to_string(),
                last_name: "Simpson".to_string(),
                bio: Some("I like donuts".to_string()),
                address: Some(Address {
                    street: "742 Evergreen Terrace".to_string(),
                    city: "Springfield".to_string(),
                    state: "Illinois".to_string(),
                    zip: "62701".to_string(),
                    country: "USA".to_string(),
                }),
                created_at: Default::default(),
                updated_at: Default::default(),
            }
        }
    }
}
