use std::time::Duration;

use crate::args::Args;
use openssl::ssl::{SslContextBuilder, SslMethod, SslVerifyMode};
use scylla::{Session, SessionBuilder};

pub async fn initialize_session(args: &Args) -> Session {
    let mut builder = SessionBuilder::new()
        .known_node(&args.host)
        .use_keyspace(&args.keyspace, false)
        .connection_timeout(Duration::from_secs(args.timeout));

    if let (Some(user), Some(password)) = (&args.user, &args.password) {
        builder = builder.user(user, password);
    }

    if let Some(ca) = &args.ca {
        let mut context_builder = SslContextBuilder::new(SslMethod::tls())
            .map_err(|e| {
                eprintln!("Failed to create SSL context: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        context_builder
            .set_ca_file(ca)
            .map_err(|e| {
                eprintln!("Failed to set CA file: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        context_builder.set_verify(SslVerifyMode::PEER);

        if let Some(key) = &args.cert {
            context_builder
                .set_certificate_file(key, openssl::ssl::SslFiletype::PEM)
                .map_err(|e| {
                    eprintln!("Failed to set certificate file: {}", e);
                    std::process::exit(1);
                })
                .unwrap();

            let key = args
                .key
                .as_ref()
                .expect("Private key file is required when certificate is provided");
            context_builder
                .set_private_key_file(key, openssl::ssl::SslFiletype::PEM)
                .map_err(|e| {
                    eprintln!("Failed to set private key file: {}", e);
                    std::process::exit(1);
                })
                .unwrap();
        }

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
