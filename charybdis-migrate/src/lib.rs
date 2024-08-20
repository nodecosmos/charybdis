use crate::args::Args;
use crate::migration::Migration;
use crate::session::initialize_session;
use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use scylla::Session;

pub mod args;
pub mod migration;
pub mod model;
pub mod session;

pub struct MigrationBuilder {
    pub(crate) args: Args,
}

impl MigrationBuilder {
    pub fn new() -> Self {
        Self { args: Args::default() }
    }

    pub async fn build(self) -> Migration {
        let session: Session = initialize_session(&self.args).await;
        let current_db_schema = DbSchema::new(&session, self.args.keyspace.clone()).await;
        let current_code_schema = CodeSchema::new(&self.args.project_root);

        let migration = Migration::new(current_db_schema, current_code_schema, session, self.args);

        migration
    }

    pub fn host(mut self, host: &str) -> Self {
        self.args.host = host.to_string();
        self
    }

    pub fn keyspace(mut self, keyspace: &str) -> Self {
        self.args.keyspace = keyspace.to_string();
        self
    }

    pub fn user(mut self, user: Option<&str>) -> Self {
        self.args.user = user.map(|s| s.to_string());
        self
    }

    pub fn password(mut self, password: Option<&str>) -> Self {
        self.args.password = password.map(|s| s.to_string());
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.args.timeout = timeout;
        self
    }

    pub fn drop_and_replace(mut self, drop_and_replace: bool) -> Self {
        self.args.drop_and_replace = drop_and_replace;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.args.verbose = verbose;
        self
    }

    pub fn ca(mut self, ca: Option<&str>) -> Self {
        self.args.ca = ca.map(|s| s.to_string());
        self
    }

    pub fn cert(mut self, cert: Option<&str>) -> Self {
        self.args.cert = cert.map(|s| s.to_string());
        self
    }

    pub fn key(mut self, key: Option<&str>) -> Self {
        self.args.key = key.map(|s| s.to_string());
        self
    }
}

impl From<Args> for MigrationBuilder {
    fn from(args: Args) -> Self {
        Self { args }
    }
}

impl Default for MigrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
