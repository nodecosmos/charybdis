use crate::errors::AppError;
use crate::models::materialized_views::PostsByCreator;
use crate::models::post::{Post, UpdateArchivedPost, UpdateDescriptionPost};
use actix_web::{get, post, put, web, HttpResponse};
use charybdis::batch::ModelBatch;
use charybdis::operations::{Find, InsertWithCallbacks, UpdateWithCallbacks};
use charybdis::types::{Timestamp, Uuid};
use futures::StreamExt;
use scylla::client::caching_session::CachingSession;

// In this sample params from path will be mapped to struct fields, which makes lookup by primary key easier.
// Note that we use camelCase for field names, as we have #[serde(rename_all = "camelCase")] in Post struct.
#[get("/{communityId}/{createdAt}/{id}")]
pub async fn get_post(db_session: web::Data<CachingSession>, post: web::Path<Post>) -> Result<HttpResponse, AppError> {
    let post = post.find_by_primary_key().execute(&db_session).await?;

    Ok(HttpResponse::Ok().json(post))
}

// Her we utilize builtin functions to get post by primary key
#[get("/{community_id}/{created_at}/{id}/builtin_fns")]
pub async fn get_post_builtin(
    db_session: web::Data<CachingSession>,
    params: web::Path<(Uuid, Timestamp, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (community_id, created_at, id) = params.into_inner();

    // for up to 3 primary keys we have automatically generated find_by_... functions
    let post = Post::find_by_community_id_and_created_at_and_id(community_id, created_at, id)
        .execute(&db_session)
        .await?;

    Ok(HttpResponse::Ok().json(post))
}

#[get("/{community_id}/community_posts")]
pub async fn get_community_posts(
    db_session: web::Data<CachingSession>,
    community_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let posts = Post::find_by_community_id(*community_id)
        .execute(&db_session)
        .await?
        .try_collect()
        .await?;

    Ok(HttpResponse::Ok().json(posts))
}

#[post("")]
pub async fn create_post(
    db_session: web::Data<CachingSession>,
    mut post: web::Json<Post>,
) -> Result<HttpResponse, AppError> {
    post.insert_cb(&()).execute(&db_session).await?;

    Ok(HttpResponse::Created().json(post))
}

#[get("/{user_id}/user_posts")]
pub async fn get_user_posts(
    db_session: web::Data<CachingSession>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let posts = PostsByCreator::find_by_partition_key_value((*user_id,))
        .execute(&db_session)
        .await?
        .try_collect()
        .await?;

    Ok(HttpResponse::Ok().json(posts))
}

#[put("/update_description")]
pub async fn update_post_description(
    db_session: web::Data<CachingSession>,
    post: web::Json<UpdateDescriptionPost>,
) -> Result<HttpResponse, AppError> {
    let mut post = post.into_inner();
    post.update_cb(&()).execute(&db_session).await?;

    Ok(HttpResponse::Ok().json(post))
}

#[put("/archive_community_posts/{community_id}")]
pub async fn archive_community_posts(
    db_session: web::Data<CachingSession>,
    community_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let mut posts = UpdateArchivedPost::find_by_community_id(*community_id)
        .execute(&db_session)
        .await?;

    let mut batch = UpdateArchivedPost::batch();

    while let Some(post) = posts.next().await {
        let mut post = post?;
        post.is_archived = true;
        batch.append_update_owned(post);
    }

    batch.execute(&db_session).await?;

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_session::db_session;
    use crate::test::tests::init_app;
    use actix_web::test;

    #[actix_web::test]
    async fn create_post() {
        let app = init_app().await;
        let post = Post::sample();

        let create_req = test::TestRequest::post().uri("/posts").set_json(&post).to_request();
        let create_resp = test::call_service(&app, create_req).await;

        assert_eq!(create_resp.status(), actix_web::http::StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn get_post() {
        let app = init_app().await;
        let db = db_session().await;
        let mut post = Post::sample();

        post.insert_cb(&()).execute(&db).await.unwrap();

        let req = test::TestRequest::get()
            .uri(&format!(
                "/posts/{}/{}/{}",
                post.community_id,
                post.created_at.to_rfc3339(),
                post.id
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let fetched_post: Post = test::read_body_json(resp).await;

        assert_eq!(fetched_post.id, post.id);
        assert_eq!(fetched_post.title, post.title);
    }

    #[actix_web::test]
    async fn get_community_posts() {
        let app = init_app().await;
        let community_id = Uuid::new_v4();
        Post::populate_sample_posts_per_partition(community_id).await;

        let req = test::TestRequest::get()
            .uri(&format!("/posts/{}/community_posts", community_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let posts: Vec<Post> = test::read_body_json(resp).await;

        assert_eq!(posts.len(), 32);
    }

    #[actix_web::test]
    async fn get_user_posts() {
        let app = init_app().await;
        let db = db_session().await;
        let mut post = Post::sample();

        post.insert_cb(&()).execute(&db).await.unwrap();

        let req = test::TestRequest::get()
            .uri(&format!("/posts/{}/user_posts", post.creator_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let posts: Vec<PostsByCreator> = test::read_body_json(resp).await;

        assert_eq!(posts.len(), 1);
    }

    #[actix_web::test]
    async fn update_post_description() {
        let app = init_app().await;
        let db = db_session().await;
        let mut post = Post::sample();

        post.insert_cb(&()).execute(&db).await.unwrap();
        post.description = "New Description".to_string();

        let req = test::TestRequest::put()
            .uri("/posts/update_description")
            .set_json(&post)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let post = post.find_by_primary_key().execute(&db).await.unwrap();

        assert_eq!(post.description, "New Description");
    }

    #[actix_web::test]
    async fn archive_community_posts() {
        let app = init_app().await;
        let db = db_session().await;
        let community_id = Uuid::new_v4();
        Post::populate_sample_posts_per_partition(community_id).await;

        let req = test::TestRequest::put()
            .uri(&format!("/posts/archive_community_posts/{}", community_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let posts = Post::find_by_community_id(community_id)
            .execute(&db)
            .await
            .unwrap()
            .try_collect()
            .await
            .unwrap();

        for post in posts {
            assert_eq!(post.is_archived, true);
        }
    }
}
