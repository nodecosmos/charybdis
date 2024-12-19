use scylla::{CachingSession, SessionBuilder};
use tokio::sync::OnceCell;

const NODE: &str = "127.0.0.1:9042";
const KEYSPACE: &str = "charybdis";
const CACHE_SIZE: usize = 1000;

static ONCE: OnceCell<CachingSession> = OnceCell::const_new();

pub async fn db_session() -> &'static CachingSession {
    ONCE.get_or_init(|| async {
        let db_session = SessionBuilder::new()
            .known_node(NODE)
            .use_keyspace(KEYSPACE, false)
            .build()
            .await
            .expect("Unable to connect to scylla hosts");
        CachingSession::from(db_session, CACHE_SIZE)
    })
    .await
}
