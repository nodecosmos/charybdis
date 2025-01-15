#[cfg(test)]
pub mod tests {
    use crate::api::*;
    use crate::db_session::db_session;
    use actix_http::Request;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::{test, web, App, Error};
    use std::sync::Arc;

    pub async fn init_app() -> impl Service<Request, Response = ServiceResponse, Error = Error> {
        let db_session = db_session().await;
        let db_session_arc = Arc::new(db_session);

        // Initialize Actix App
        test::init_service(
            App::new()
                .app_data(web::Data::from(db_session_arc.clone()))
                .service(
                    web::scope("/users")
                        .service(get_user)
                        .service(get_user_by_username)
                        .service(create_user),
                )
                .service(
                    web::scope("/communities")
                        .service(get_community)
                        .service(create_community),
                )
                .service(
                    web::scope("/posts")
                        .service(create_post)
                        .service(get_post)
                        .service(get_community_posts)
                        .service(get_user_posts)
                        .service(update_post_description)
                        .service(archive_community_posts),
                ),
        )
        .await
    }
}
