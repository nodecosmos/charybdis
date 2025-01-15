use charybdis::macros::charybdis_view_model;
use charybdis::types::{Text, Timestamp, Uuid};
use serde::{Deserialize, Serialize};

#[charybdis_view_model(
    table_name=posts_by_creator,
    base_table=posts,
    partition_keys=[creator_id],
    clustering_keys=[community_id, created_at, id]
)]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostsByCreator {
    pub creator_id: Uuid,
    pub community_id: Uuid,
    pub created_at: Timestamp,
    pub id: Uuid,
    pub title: Text,
}
