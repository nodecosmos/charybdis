use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{env, io};

use clap::Parser;
use scylla::Session;

use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use session::initialize_session;

use crate::migration::Migration;

mod migration;
mod model;

mod session;

/// Automatic Migration Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Scylla Host
    #[arg(long)]
    host: String,

    /// Keyspace
    #[arg(short, long)]
    keyspace: String,

    #[arg(short, long, default_value = None)]
    user: Option<String>,

    #[arg(short, long, default_value = None)]
    password: Option<String>,

    #[arg(short, long, default_value_t = 30)]
    timeout: u64,

    /// Drop and recreate columns in case of type change
    #[arg(short, long, default_value_t = false)]
    drop_and_replace: bool,

    /// Prints alter table options queries
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Path to the certificate file if using SSL
    #[arg(long, default_value = None)]
    cert: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let project_root = get_project_root().unwrap();

    let session: Session = initialize_session(&args).await;

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
