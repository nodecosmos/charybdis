use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{env, io};

use clap::Parser;
use scylla::Session;

use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use migrate::Args;
use migrate::session::initialize_session;

use migrate::migration::Migration;


/// Automatic Migration Tool
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let project_root = get_project_root().unwrap();
    let session: Session = initialize_session(&args).await;

    if env::var("FORCE_COLOR").is_ok() {
        colored::control::set_override(true);
    }

    let current_db_schema = DbSchema::new(&session, args.keyspace.clone()).await;
    let current_code_schema = CodeSchema::new(&project_root);

    let migration = Migration::new(&current_db_schema, &current_code_schema, &session, &args);

    migration.run().await;

    DbSchema::new(&session, args.keyspace)
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
