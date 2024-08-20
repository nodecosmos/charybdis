use clap::Parser;
use migrate::args::Args;
use migrate::session::initialize_session;
use migrate::MigrationBuilder;
use scylla::Session;
use std::env;

/// Automatic Migration Tool
#[tokio::main]
async fn main() {
    if env::var("FORCE_COLOR").is_ok() {
        colored::control::set_override(true);
    }

    let args = Args::parse();
    let session: Session = initialize_session(&args).await;
    let migration = MigrationBuilder::from(args).build(&session).await;

    migration.run().await;
    migration.write_schema_to_json().await;
}
