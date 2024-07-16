use std::time::Duration;

use openssl::ssl::{SslContextBuilder, SslMethod, SslVerifyMode};
use scylla::{Session, SessionBuilder};

use crate::Args;

pub(crate) async fn initialize_session(args: &Args) -> Session {
    let mut builder = SessionBuilder::new()
        .known_node(&args.host)
        .use_keyspace(&args.keyspace, false)
        .connection_timeout(Duration::from_secs(args.timeout));

    if let (Some(user), Some(password)) = (&args.user, &args.password) {
        builder = builder.user(user, password);
    }

    if let Some(cert) = &args.cert {
        let mut context_builder = SslContextBuilder::new(SslMethod::tls())
            .map_err(|e| {
                eprintln!("Failed to create SSL context: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        context_builder
            .set_ca_file(cert)
            .map_err(|e| {
                eprintln!("Failed to set CA file: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        context_builder.set_verify(SslVerifyMode::PEER);

        builder = builder.ssl_context(Some(context_builder.build()));
    }

    builder
        .build()
        .await
        .map_err(|e| {
            eprintln!("Failed to create session: {}", e);
            std::process::exit(1);
        })
        .unwrap()
}
