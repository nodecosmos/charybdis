use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{env, io};

use clap::Parser;
use scylla::Session;
use charybdis_parser::schema::db_schema::DbSchema;
use migrate::args::Args;
use migrate::MigrationBuilder;
use migrate::session::initialize_session;

/// Automatic Migration Tool
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let project_root = get_project_root().unwrap();
    let migration_data = MigrationBuilder::from_args(args);
    let keyspace = migration_data.keyspace.clone();
    let session: Session = initialize_session(&migration_data).await;

    if env::var("FORCE_COLOR").is_ok() {
        colored::control::set_override(true);
    }

    migration_data.run_with_session(&session, &project_root).await;

    DbSchema::new(&session, keyspace)
        .await
        .write_schema_to_json(project_root);
}

pub(crate) fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let path_ancestors = path.as_path().ancestors();

    for p in path_ancestors {
        let has_cargo = read_dir(p)?.any(|p| p.unwrap().file_name() == *"Cargo.lock");
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }

    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}
