use crate::errors::AppError;
use crate::models::community::Community;
use actix_web::{get, post, web, HttpResponse};
use charybdis::operations::{Find, InsertWithCallbacks};
use charybdis::scylla::CachingSession;
use charybdis::types::Uuid;

#[get("/{id}")]
pub async fn get_community(
    db_session: web::Data<CachingSession>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let community = Community::find_by_primary_key_value((*id,))
        .execute(&db_session)
        .await?;

    Ok(HttpResponse::Ok().json(community))
}

#[post("")]
pub async fn create_community(
    db_session: web::Data<CachingSession>,
    mut community: web::Json<Community>,
) -> Result<HttpResponse, AppError> {
    community.insert_cb(&()).execute(&db_session).await?;

    Ok(HttpResponse::Created().json(community))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_session::db_session;
    use crate::test::tests::init_app;
    use actix_web::test;

    #[actix_web::test]
    async fn get_community() {
        let app = init_app().await;
        let db = db_session().await;
        let mut community = Community::sample();

        community.insert_cb(&()).execute(&db).await.unwrap();

        let req = test::TestRequest::get()
            .uri(&format!("/communities/{}", community.id))
            .to_request();

        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), 200);

        let fetched_community: Community = test::read_body_json(res).await;

        assert_eq!(fetched_community.id, community.id);
        assert_eq!(fetched_community.title, community.title);
        assert_eq!(
            fetched_community.created_at.timestamp(),
            community.created_at.timestamp()
        );
        assert_eq!(
            fetched_community.updated_at.timestamp(),
            community.updated_at.timestamp()
        );
    }

    #[actix_web::test]
    async fn create_community() {
        let app = init_app().await;
        let community = Community::sample();

        let req = test::TestRequest::post()
            .uri("/communities")
            .set_json(&community)
            .to_request();

        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), 201);

        let created_community: Community = test::read_body_json(res).await;

        assert_eq!(created_community.title, community.title);
        assert_eq!(created_community.description, community.description);
    }
}
