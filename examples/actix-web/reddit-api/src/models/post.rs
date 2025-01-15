use crate::errors::AppError;
use crate::models::udts::Profile;
use charybdis::callbacks::Callbacks;
use charybdis::macros::charybdis_model;
use charybdis::scylla::CachingSession;
use charybdis::types::{Text, Timestamp, Uuid};
use serde::{Deserialize, Serialize};

#[charybdis_model(
    table_name = posts,
    partition_keys = [community_id],
    clustering_keys = [created_at, id],
    global_secondary_indexes = [],
)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Post {
    pub community_id: Uuid,
    pub created_at: Timestamp,
    pub id: Uuid,
    pub title: Text,
    pub description: Text,
    pub updated_at: Timestamp,
    pub creator_id: Uuid,
    pub creator: Profile,
    pub is_archived: bool,
}

impl Callbacks for Post {
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

partial_post!(
    UpdateDescriptionPost,
    community_id,
    created_at,
    updated_at,
    id,
    description
);

impl Callbacks for UpdateDescriptionPost {
    type Extension = ();
    type Error = AppError;

    async fn before_update(&mut self, _: &CachingSession, _: &Self::Extension) -> Result<(), AppError> {
        self.updated_at = chrono::Utc::now();

        Ok(())
    }
}

partial_post!(
    UpdateArchivedPost,
    community_id,
    created_at,
    updated_at,
    id,
    is_archived
);

#[cfg(test)]
mod tests {
    use crate::db_session::db_session;
    use crate::models::post::Post;
    use crate::models::udts::Profile;
    use charybdis::batch::ModelBatch;
    use charybdis::types::Uuid;
    use chrono::Utc;

    impl Post {
        pub fn sample() -> Post {
            Post {
                community_id: Uuid::new_v4(),
                created_at: Utc::now(),
                id: Uuid::new_v4(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                updated_at: Default::default(),
                creator_id: Uuid::new_v4(),
                creator: Profile::sample(),
                is_archived: false,
            }
        }

        pub async fn populate_sample_posts_per_partition(community_id: Uuid) {
            let db_session = db_session().await;
            let mut posts = vec![];
            for i in 0..32 {
                let mut post = Post::sample();
                post.community_id = community_id;
                post.title = format!("Post {}", i);
                post.description = format!("Post {}", i);
                post.creator_id = Uuid::new_v4();
                posts.push(post);
            }

            Post::batch()
                .chunked_insert(&db_session, &posts, 100)
                .await
                .expect("Failed to insert posts");
        }
    }
}
