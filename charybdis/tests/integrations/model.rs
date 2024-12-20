use crate::common::db_session;
use charybdis::batch::ModelBatch;
use charybdis::errors::CharybdisError;
use charybdis::model::{BaseModel, Model};
use charybdis::stream::CharybdisModelStream;
use charybdis::types::{Boolean, Int, Text, Uuid};
use charybdis_macros::{charybdis_model, charybdis_udt_model, charybdis_view_model};

#[derive(Debug, Default, Clone, PartialEq)]
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
    global_secondary_indexes = [username],
)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub username: Text,
    pub password: Text,
    pub email: Text,
    pub first_name: Text,
    pub last_name: Text,
    pub bio: Option<Text>,
    pub address: Option<Address>,
    pub is_confirmed: Boolean,
}

partial_user!(UpdateUsernameUser, id, username);

impl User {
    pub async fn populate_sample_users() {
        let db_session = db_session().await;
        let users = (0..32)
            .map(|i| {
                let id = Uuid::new_v4();
                let mut new_user = User::homer(id);
                new_user.username = format!("user_{}", i);
                new_user.email = format!("user_{}@gmail.com", i);

                new_user
            })
            .collect::<Vec<User>>();

        User::batch()
            .chunked_insert(&db_session, &users, 100)
            .await
            .expect("Failed to insert users");
    }

    pub fn homer(id: Uuid) -> Self {
        User {
            id,
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
            is_confirmed: true,
        }
    }
}

#[tokio::test]
async fn user_model_queries() {
    assert_eq!(User::DB_MODEL_NAME, "users");
    assert_eq!(
        User::FIND_ALL_QUERY,
        "SELECT id, username, password, email, first_name, last_name, bio, address, is_confirmed FROM users"
    );
    assert_eq!(
        User::FIND_BY_PRIMARY_KEY_QUERY,
        "SELECT id, username, password, email, \
         first_name, last_name, bio, address, is_confirmed FROM users WHERE id = ?"
    );
    assert_eq!(
        User::FIND_BY_PARTITION_KEY_QUERY,
        "SELECT id, username, password, email, \
         first_name, last_name, bio, address, is_confirmed FROM users WHERE id = ?"
    );
    assert_eq!(
        User::FIND_FIRST_BY_PARTITION_KEY_QUERY,
        "SELECT id, username, password, email, \
         first_name, last_name, bio, address, is_confirmed FROM users WHERE id = ? LIMIT 1"
    );
    assert_eq!(
        User::INSERT_QUERY,
        "INSERT INTO users (id, username, password, email, first_name, last_name, bio, address, is_confirmed) \
         VALUES (:id, :username, :password, :email, :first_name, :last_name, :bio, :address, :is_confirmed)"
    );
    assert_eq!(
        User::INSERT_IF_NOT_EXIST_QUERY,
        "INSERT INTO users (id, username, password, email, first_name, last_name, bio, address, is_confirmed) \
         VALUES (:id, :username, :password, :email, :first_name, :last_name, :bio, :address, :is_confirmed) \
         IF NOT EXISTS"
    );
    assert_eq!(
        User::UPDATE_QUERY,
        "UPDATE users SET username = :username, password = :password, email = :email, first_name = :first_name, \
        last_name = :last_name, bio = :bio, address = :address, is_confirmed = :is_confirmed WHERE id = :id"
    );
    assert_eq!(User::DELETE_QUERY, "DELETE FROM users WHERE id = ?");
    assert_eq!(User::DELETE_BY_PARTITION_KEY_QUERY, "DELETE FROM users WHERE id = ?");
}

#[charybdis_view_model(
    table_name=user_by_email,
    base_table=users,
    partition_keys=[email],
    clustering_keys=[id]
)]
pub struct UserByEmail {
    pub id: Uuid,
    pub email: Text,
    pub username: Text,
}

#[charybdis_model(
    table_name = posts,
    partition_keys = [category_id],
    clustering_keys = [order_idx, title],
    global_secondary_indexes = [author_id],
    local_secondary_indexes = [title],
)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Post {
    pub category_id: Uuid,
    pub order_idx: Int,
    pub title: Text,
    pub content: Text,
    pub author_id: Uuid,
}

impl Post {
    pub async fn populate_sample_posts_per_partition(category_id: Uuid) {
        let db_session = db_session().await;
        let posts = (0..32)
            .map(|i| Post {
                category_id,
                order_idx: i,
                title: format!("Post {}", i),
                content: "Lorem ipsum dolor sit amet".to_string(),
                author_id: Uuid::new_v4(),
            })
            .collect::<Vec<Post>>();

        Post::batch()
            .chunked_insert(&db_session, &posts, 100)
            .await
            .expect("Failed to insert posts");
    }
}

#[tokio::test]
async fn post_model_queries() {
    assert_eq!(Post::DB_MODEL_NAME, "posts");
    assert_eq!(
        Post::FIND_ALL_QUERY,
        "SELECT category_id, order_idx, title, content, author_id FROM posts"
    );
    assert_eq!(
        Post::FIND_BY_PRIMARY_KEY_QUERY,
        "SELECT category_id, order_idx, title, content, author_id FROM posts \
        WHERE category_id = ? AND order_idx = ? AND title = ?"
    );
    assert_eq!(
        Post::FIND_BY_PARTITION_KEY_QUERY,
        "SELECT category_id, order_idx, title, content, author_id FROM posts WHERE category_id = ?"
    );
    assert_eq!(
        Post::FIND_FIRST_BY_PARTITION_KEY_QUERY,
        "SELECT category_id, order_idx, title, content, author_id FROM posts WHERE category_id = ? LIMIT 1"
    );
    assert_eq!(
        Post::INSERT_QUERY,
        "INSERT INTO posts (category_id, order_idx, title, content, author_id) \
         VALUES (:category_id, :order_idx, :title, :content, :author_id)"
    );
    assert_eq!(
        Post::INSERT_IF_NOT_EXIST_QUERY,
        "INSERT INTO posts (category_id, order_idx, title, content, author_id) \
         VALUES (:category_id, :order_idx, :title, :content, :author_id) \
         IF NOT EXISTS"
    );
    assert_eq!(
        Post::UPDATE_QUERY,
        "UPDATE posts SET content = :content, author_id = :author_id \
        WHERE category_id = :category_id AND order_idx = :order_idx AND title = :title"
    );
    assert_eq!(
        Post::DELETE_QUERY,
        "DELETE FROM posts WHERE category_id = ? AND order_idx = ? AND title = ?"
    );
    assert_eq!(
        Post::DELETE_BY_PARTITION_KEY_QUERY,
        "DELETE FROM posts WHERE category_id = ?"
    );
}

#[tokio::test]
async fn find_various() -> Result<(), CharybdisError> {
    let category_id = Uuid::new_v4();
    let db_session = &db_session().await;

    Post::populate_sample_posts_per_partition(category_id).await;

    let posts: CharybdisModelStream<Post> = Post::find_by_category_id(category_id).execute(db_session).await?;
    assert_eq!(posts.try_collect().await?.len(), 32);

    let posts: CharybdisModelStream<Post> = Post::find_by_category_id_and_order_idx(category_id, 1)
        .execute(db_session)
        .await?;
    assert_eq!(posts.try_collect().await?.len(), 1);

    let posts: Post = Post::find_by_category_id_and_order_idx_and_title(category_id, 1, "Post 1".to_string())
        .execute(db_session)
        .await?;
    assert_eq!(posts.title, "Post 1");

    let post: Post = Post::find_first_by_category_id(category_id).execute(db_session).await?;
    assert_eq!(post.order_idx, 0);

    let post: Post = Post::find_first_by_category_id_and_order_idx(category_id, 1)
        .execute(db_session)
        .await?;
    assert_eq!(post.title, "Post 1");

    let maybe_post: Option<Post> = Post::maybe_find_first_by_category_id(Uuid::new_v4())
        .execute(db_session)
        .await?;

    assert!(maybe_post.is_none());

    let maybe_post: Option<Post> = Post::maybe_find_first_by_category_id_and_order_idx(category_id, 1)
        .execute(db_session)
        .await?;
    assert!(maybe_post.is_some());

    let maybe_post: Option<Post> =
        Post::maybe_find_first_by_category_id_and_order_idx_and_title(category_id, 2, "Post 2".to_string())
            .execute(db_session)
            .await?;
    assert!(maybe_post.is_some());

    // find by local secondary index
    let posts: CharybdisModelStream<Post> = Post::find_by_category_id_and_title(category_id, "Post 2".to_string())
        .execute(db_session)
        .await?;
    assert!(posts.try_collect().await?.len() > 0);

    let post: Post = Post::find_first_by_category_id_and_title(category_id, "Post 2".to_string())
        .execute(db_session)
        .await?;
    assert_eq!(post.title, "Post 2");

    let maybe_post: Option<Post> = Post::maybe_find_first_by_category_id_and_title(category_id, "Post 42".to_string())
        .execute(db_session)
        .await?;
    assert!(maybe_post.is_none());

    let author_id = post.author_id;

    // find by global secondary index
    let posts: CharybdisModelStream<Post> = Post::find_by_author_id(author_id).execute(db_session).await?;
    assert!(posts.try_collect().await?.len() > 0);

    let post: Post = Post::find_first_by_author_id(author_id).execute(db_session).await?;
    assert_eq!(post.author_id, author_id);

    let post: Option<Post> = Post::maybe_find_first_by_author_id(Uuid::new_v4())
        .execute(db_session)
        .await?;
    assert!(post.is_none());

    Ok(())
}
