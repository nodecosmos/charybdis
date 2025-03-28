use crate::args::Args;
use crate::model::data::ModelData;
use crate::model::{ModelMigration, ModelType};
use colored::Colorize;
use scylla::client::session::Session;

use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use charybdis_parser::schema::SchemaObject;

pub struct Migration<'a> {
    current_db_schema: DbSchema,
    current_code_schema: CodeSchema,
    session: &'a Session,
    args: Args,
}

impl<'a> Migration<'a> {
    pub fn new(current_db_schema: DbSchema, current_code_schema: CodeSchema, session: &'a Session, args: Args) -> Self {
        Migration {
            current_db_schema,
            current_code_schema,
            session,
            args,
        }
    }

    pub async fn run(&self) {
        self.run_udts().await;
        self.run_tables().await;
        self.run_materialized_views().await;

        println!("\n{}", "Migration plan ran successfully!".bright_green());
    }

    pub async fn write_schema_to_json(&self) {
        DbSchema::new(self.session, self.args.keyspace.clone())
            .await
            .write_schema_to_json(&self.args.current_dir);
    }

    pub fn get_code_schema(&self) -> &CodeSchema {
        &self.current_code_schema
    }

    pub fn get_db_schema(&self) -> &DbSchema {
        &self.current_db_schema
    }

    async fn run_udts(&self) {
        let empty_udt = SchemaObject::default();

        for (name, code_udt_schema) in self.current_code_schema.udts.iter() {
            let model_data = ModelData::new(
                name,
                ModelType::Udt,
                code_udt_schema,
                self.current_db_schema.udts.get(name).unwrap_or(&empty_udt),
            );

            let migration = ModelMigration::new(&model_data, self.session, &self.args);

            migration.run().await;
        }
    }

    async fn run_tables(&self) {
        let empty_table = SchemaObject::default();

        for (name, code_table_schema) in self.current_code_schema.tables.iter() {
            let model_data = ModelData::new(
                name,
                ModelType::Table,
                code_table_schema,
                self.current_db_schema.tables.get(name).unwrap_or(&empty_table),
            );

            let migration = ModelMigration::new(&model_data, self.session, &self.args);

            migration.run().await;
        }
    }

    async fn run_materialized_views(&self) {
        let empty_mv = SchemaObject::default();

        for (name, code_mv_schema) in self.current_code_schema.materialized_views.iter() {
            let model_data = ModelData::new(
                name,
                ModelType::MaterializedView,
                code_mv_schema,
                self.current_db_schema.materialized_views.get(name).unwrap_or(&empty_mv),
            );

            let migration = ModelMigration::new(&model_data, self.session, &self.args);

            migration.run().await;
        }
    }
}
