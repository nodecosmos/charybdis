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

impl Default for MigrationBuilder {
    fn default() -> Self {
        Self::new()
    }
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

        let current_db_schema = DbSchema::new(session, self.args.keyspace.clone()).await;
        let current_code_schema: CodeSchema = self.args.code_schema_override_json.as_ref()
            .map(|json| serde_json::from_str(json).unwrap())
            .unwrap_or_else(|| CodeSchema::new(&self.args.current_dir));

        let migration = Migration::new(current_db_schema, current_code_schema, session, self.args);

        migration
    }

    pub fn keyspace(mut self, keyspace: String) -> Self {
        self.args.keyspace = keyspace;
        self
    }

    pub fn current_dir(mut self, current_dir: String) -> Self {
        self.args.current_dir = current_dir;
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

    pub fn code_schema_override_json(mut self, code_schema_override_json: String) -> Self {
        self.args.code_schema_override_json = Some(code_schema_override_json);
        self
    }
}

impl From<Args> for MigrationBuilder {
    fn from(args: Args) -> Self {
        Self { args }
    }
}
