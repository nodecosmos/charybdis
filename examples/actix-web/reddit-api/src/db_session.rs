use scylla::{CachingSession, Session, SessionBuilder};

const NODES: [&str; 3] = ["127.0.0.1:9042", "127.0.0.1:9043", "127.0.0.1:9044"];
const KEYSPACE: &str = "reddit_example";
const CACHE_SIZE: usize = 1000;

/// For test use only; do not reinitialize in production.
pub async fn db_session() -> CachingSession {
    let db_session = default_session().await;

    CachingSession::from(db_session, CACHE_SIZE)
}

pub async fn default_session() -> Session {
    SessionBuilder::new()
        .known_nodes(NODES)
        .use_keyspace(KEYSPACE, false)
        .build()
        .await
        .expect("Unable to connect to scylla hosts")
}
