use crate::errors::AppError;
use crate::models::udts::Profile;
use charybdis::callbacks::Callbacks;
use charybdis::macros::charybdis_model;
use charybdis::types::{Text, Timestamp, Uuid};
use scylla::client::caching_session::CachingSession;
use serde::{Deserialize, Serialize};

#[charybdis_model(
    table_name = communities,
    partition_keys = [id],
    clustering_keys = [],
    global_secondary_indexes = [],
)]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Community {
    pub id: Uuid,
    pub title: Text,
    pub description: Text,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub creator_id: Uuid,
    pub creator: Profile,
}

impl Callbacks for Community {
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

#[cfg(test)]
impl Community {
    pub fn sample() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Sample Community".into(),
            description: "Sample Community Description".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            creator_id: Uuid::new_v4(),
            creator: Profile::sample(),
        }
    }
}
