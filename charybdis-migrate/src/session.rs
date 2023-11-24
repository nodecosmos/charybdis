use crate::Args;
use scylla::{Session, SessionBuilder};
use std::time::Duration;

pub(crate) async fn initialize_session(args: &Args) -> Session {
    let mut builder = SessionBuilder::new()
        .known_node(&args.host)
        .use_keyspace(&args.keyspace, false)
        .connection_timeout(Duration::from_secs(args.timeout));

    if !args.user.is_empty() {
        builder = builder.user(&args.user, &args.password);
    }

    builder.build().await.unwrap()
}
