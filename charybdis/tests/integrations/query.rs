use crate::common::db_session;
use crate::custom_fields::AddressTypeCustomField;
use crate::model::{Post, User, SAMPLE_MODEL_COUNT};
use charybdis::batch::ModelBatch;
use charybdis::errors::CharybdisError;
use charybdis::operations::{Delete, Find, Insert, Update};
use charybdis::scylla::PagingStateResponse;
use charybdis::stream::CharybdisModelStream;

#[tokio::test]
async fn model_mutation() {
    let id = uuid::Uuid::new_v4();
    let new_user = User::homer(id);

    let db_session = db_session().await;

    new_user
        .insert()
        .execute(&db_session)
        .await
        .expect("Failed to insert user");

    let mut user = User::find_by_id(id)
        .execute(&db_session)
        .await
        .expect("Failed to find user");

    assert_eq!(user, new_user);

    user.bio = Some("I like beer".to_string());
    user.address.as_mut().expect("homer should have address").addr_type = AddressTypeCustomField::HomeAddress;
    let tag = ("Second Key".to_string(), "Second Value".to_string());
    user.user_extra_data.user_tags.push(tag.clone());

    user.update().execute(&db_session).await.expect("Failed to update user");

    let user = User::find_by_id(id)
        .execute(&db_session)
        .await
        .expect("Failed to find user");

    assert_eq!(user.bio, Some("I like beer".to_string()));
    assert_eq!(
        user.address.as_ref().expect("homer should have address").addr_type,
        AddressTypeCustomField::HomeAddress
    );
    assert_eq!(user.user_extra_data.user_tags.last().cloned(), Some(tag));

    user.delete().execute(&db_session).await.expect("Failed to delete user");
}

#[tokio::test]
async fn model_row() {
    let id = uuid::Uuid::new_v4();
    let new_user = User::homer(id);

    let db_session = db_session().await;

    new_user
        .insert()
        .execute(&db_session)
        .await
        .expect("Failed to insert user");

    let user = User::find_by_id(id)
        .execute(&db_session)
        .await
        .expect("Failed to find user");

    assert_eq!(user, new_user);

    user.delete().execute(&db_session).await.expect("Failed to delete user");
}

#[tokio::test]
async fn optional_model_row() {
    let id = uuid::Uuid::new_v4();
    let db_session = db_session().await;

    let user = User::maybe_find_first_by_id(id)
        .execute(&db_session)
        .await
        .expect("Failed to find user");

    assert_eq!(user, None);
}

#[tokio::test]
async fn model_stream() {
    let db_session = db_session().await;

    User::populate_sample_users().await;

    let users: CharybdisModelStream<User> = User::find_all()
        .execute(&db_session)
        .await
        .expect("Failed to find users");
    let users_vec = users.try_collect().await.expect("Failed to collect users");

    assert_eq!(users_vec.len(), SAMPLE_MODEL_COUNT);

    User::delete_batch()
        .chunked_delete(&db_session, &users_vec, 100)
        .await
        .expect("Failed to delete users");
}

#[tokio::test]
async fn model_paged() {
    let db_session = db_session().await;
    let category_id = uuid::Uuid::new_v4();

    Post::populate_sample_posts_per_partition(category_id, None).await;

    let (posts, paging_state_response) = Post::find_by_partition_key_value_paged((category_id,))
        .page_size(3)
        .execute(&db_session)
        .await
        .expect("Failed to find posts");

    let posts = posts
        .collect::<Result<Vec<Post>, CharybdisError>>()
        .expect("Failed to collect posts");

    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].order_idx, 0);
    assert_eq!(posts[1].order_idx, 1);
    assert_eq!(posts[2].order_idx, 2);
    assert!(
        matches!(paging_state_response, PagingStateResponse::HasMorePages { .. }),
        "Expected more pages, but got NoMorePages"
    );

    if let PagingStateResponse::HasMorePages { state } = paging_state_response {
        let (next_page_posts, paging_state_response) = Post::find_by_partition_key_value_paged((category_id,))
            .page_size(30) // 32 is the total number of posts
            .paging_state(state)
            .execute(&db_session)
            .await
            .expect("Failed to find posts");
        let next_page_posts = next_page_posts
            .collect::<Result<Vec<Post>, CharybdisError>>()
            .expect("Failed to collect posts");

        assert_eq!(next_page_posts.len(), 29);
        assert_eq!(next_page_posts[0].order_idx, 3);
        assert_eq!(next_page_posts[1].order_idx, 4);
        assert_eq!(next_page_posts[28].order_idx, 31);

        assert!(
            matches!(paging_state_response, PagingStateResponse::NoMorePages),
            "Expected no more pages, but got HasMorePages"
        );

        Post::delete_batch()
            .chunked_delete(&db_session, &next_page_posts, 100)
            .await
            .expect("Failed to delete posts");
    }

    Post::delete_batch()
        .chunked_delete(&db_session, &posts, 100)
        .await
        .expect("Failed to delete posts");
}
