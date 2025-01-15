use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod api;
mod db_session;
mod errors;
mod models;
mod test;

use crate::db_session::db_session;
use api::*;

const PORT: u16 = 3000;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            {
                let db_session = db_session().await;
                let db_session_arc = Arc::new(db_session);

                HttpServer::new(move || {
                    App::new()
                        .wrap(Logger::new("%a %r %s %b %{Referer}i %{User-Agent}i %T"))
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
                        )
                })
                .keep_alive(std::time::Duration::from_secs(60))
                .shutdown_timeout(30)
                .bind(("0.0.0.0", PORT))
                .unwrap_or_else(|e| panic!("Could not bind to port {}.\n{}", PORT, e))
                .run()
                .await
                .unwrap_or_else(|e| panic!("Could not run server to port {}.\n{}", PORT, e))
            }
        });
}
