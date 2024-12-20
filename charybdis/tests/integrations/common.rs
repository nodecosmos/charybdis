use scylla::{CachingSession, SessionBuilder};
use tokio::sync::OnceCell;

const NODES: [&str; 3] = ["127.0.0.1:9042", "127.0.0.1:9043", "127.0.0.1:9044"];
const KEYSPACE: &str = "charybdis";
const CACHE_SIZE: usize = 1000;

static ONCE: OnceCell<CachingSession> = OnceCell::const_new();

pub async fn db_session() -> &'static CachingSession {
    ONCE.get_or_init(|| async {
        let db_session = SessionBuilder::new()
            .known_nodes(NODES)
            .use_keyspace(KEYSPACE, false)
            .build()
            .await
            .expect("Unable to connect to scylla hosts");
        CachingSession::from(db_session, CACHE_SIZE)
    })
    .await
}
