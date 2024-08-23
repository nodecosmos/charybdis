use crate::args::Args;
use crate::migration::Migration;
use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use scylla::Session;

pub mod args;
pub mod migration;
pub(crate) mod model;
pub mod session;

pub struct MigrationBuilder {
    pub(crate) args: Args,
}

impl MigrationBuilder {
    pub fn new() -> Self {
        Self { args: Args::default() }
    }

    pub async fn build(mut self, session: &Session) -> Migration {
        if self.args.keyspace.is_empty() {
            // try to get the keyspace from the session
            self.args.keyspace = session
                .get_keyspace()
                .expect("No keyspace provided and no default keyspace set")
                .to_string();
        }

        let current_db_schema = DbSchema::new(&session, self.args.keyspace.clone()).await;
        let current_code_schema = CodeSchema::new(&self.args.project_root);

        let migration = Migration::new(current_db_schema, current_code_schema, session, self.args);

        migration
    }

    pub fn keyspace(mut self, keyspace: String) -> Self {
        self.args.keyspace = keyspace;
        self
    }

    pub fn project_root(mut self, project_root: String) -> Self {
        self.args.project_root = project_root;
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
}

impl From<Args> for MigrationBuilder {
    fn from(args: Args) -> Self {
        Self { args }
    }
}
