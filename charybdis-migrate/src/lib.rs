use std::path::PathBuf;
use scylla::Session;
use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use crate::args::Args;
use crate::migration::Migration;
use crate::session::initialize_session;

pub mod migration;
pub mod model;
pub mod session;
pub mod args;

pub struct MigrationBuilder {
    pub host: String,
    pub keyspace: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub timeout: u64,
    pub drop_and_replace: bool,
    pub verbose: bool,
    pub ca: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
}

impl MigrationBuilder {
    pub fn new() -> Self {
        MigrationBuilder {
            host: String::new(),
            keyspace: String::new(),
            user: None,
            password: None,
            timeout: 30,
            drop_and_replace: false,
            verbose: false,
            ca: None,
            cert: None,
            key: None,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn keyspace(mut self, keyspace: &str) -> Self {
        self.keyspace = keyspace.to_string();
        self
    }

    pub fn user(mut self, user: Option<&str>) -> Self {
        self.user = user.map(|s| s.to_string());
        self
    }

    pub fn password(mut self, password: Option<&str>) -> Self {
        self.password = password.map(|s| s.to_string());
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn drop_and_replace(mut self, drop_and_replace: bool) -> Self {
        self.drop_and_replace = drop_and_replace;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn ca(mut self, ca: Option<&str>) -> Self {
        self.ca = ca.map(|s| s.to_string());
        self
    }

    pub fn cert(mut self, cert: Option<&str>) -> Self {
        self.cert = cert.map(|s| s.to_string());
        self
    }

    pub fn key(mut self, key: Option<&str>) -> Self {
        self.key = key.map(|s| s.to_string());
        self
    }

    pub fn from_args(args: Args) -> Self {
        Self {
            host: args.host,
            keyspace: args.keyspace,
            user: args.user,
            password: args.password,
            timeout: args.timeout,
            drop_and_replace: args.drop_and_replace,
            verbose: args.verbose,
            ca: args.ca,
            cert: args.cert,
            key: args.key,
        }
    }

    pub async fn run(self, project_root: &PathBuf) {
        let session: Session = initialize_session(&self).await;

        self.run_with_session(&session, project_root).await;
    }

    /// Run migration with provided session
    pub async fn run_with_session(self, session: &Session, project_root: &PathBuf) {
        let current_db_schema = DbSchema::new(session, self.keyspace.clone()).await;
        let current_code_schema = CodeSchema::new(project_root);

        let migration = Migration::new(&current_db_schema, &current_code_schema, session, &self);

        migration.run().await;
    }
}

impl Default for MigrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
