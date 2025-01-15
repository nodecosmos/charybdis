use crate::errors::AppError;
use crate::models::user::{ShowUser, User};
use actix_web::{get, post, web, HttpResponse};
use charybdis::operations::InsertWithCallbacks;
use charybdis::scylla::CachingSession;
use charybdis::types::Uuid;

#[get("/{id}")]
pub async fn get_user(db_session: web::Data<CachingSession>, id: web::Path<Uuid>) -> Result<HttpResponse, AppError> {
    let user = ShowUser::find_by_id(*id).execute(&db_session).await?;

    Ok(HttpResponse::Ok().json(user))
}

#[get("/{username}/username")]
pub async fn get_user_by_username(
    db_session: web::Data<CachingSession>,
    username: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let user = ShowUser::find_first_by_username(username.into_inner())
        .execute(&db_session)
        .await?;

    Ok(HttpResponse::Ok().json(user))
}

#[post("")]
pub async fn create_user(
    db_session: web::Data<CachingSession>,
    mut user: web::Json<User>,
) -> Result<HttpResponse, AppError> {
    user.insert_cb(&()).execute(&db_session).await?;

    Ok(HttpResponse::Created().json(user))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_session::db_session;
    use crate::test::tests::init_app;
    use actix_web::test;
    use charybdis::operations::{Delete, New};

    #[actix_web::test]
    async fn create_user() {
        let app = init_app().await;
        let user = User::homer();

        let create_req = test::TestRequest::post().uri("/users").set_json(&user).to_request();
        let create_resp = test::call_service(&app, create_req).await;

        assert_eq!(create_resp.status(), actix_web::http::StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn get_user() {
        let app = init_app().await;
        let db = db_session().await;
        let mut user = User::homer();

        user.insert_cb(&()).execute(&db).await.unwrap();

        let req = test::TestRequest::get()
            .uri(&format!("/users/{}", user.id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let fetched_user: ShowUser = test::read_body_json(resp).await;

        assert_eq!(fetched_user.id, user.id);
        assert_eq!(fetched_user.username, user.username);
    }

    #[actix_web::test]
    async fn get_user_by_username() {
        let app = init_app().await;
        let db = db_session().await;
        let mut user = User::new();
        user.username = "homerado".to_string();
        let req = test::TestRequest::get()
            .uri(&format!("/users/{}/username", user.username))
            .to_request();

        user.insert_cb(&()).execute(&db).await.unwrap();

        let id = user.id;

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let fetched_user: ShowUser = test::read_body_json(resp).await;

        assert_eq!(fetched_user.id, id);
        assert_eq!(fetched_user.username, user.username);

        user.delete().execute(&db).await.unwrap();
    }
}
