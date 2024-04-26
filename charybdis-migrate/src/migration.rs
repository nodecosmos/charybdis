use colored::Colorize;
use scylla::Session;

use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use charybdis_parser::schema::SchemaObject;

use crate::Args;
use crate::model::{ModelMigration, ModelType};
use crate::model::data::ModelData;

pub(crate) struct Migration<'a> {
    current_db_schema: &'a DbSchema,
    current_code_schema: &'a CodeSchema,
    session: &'a Session,
    args: &'a Args,
}

impl<'a> Migration<'a> {
    pub(crate) fn new(
        current_db_schema: &'a DbSchema,
        current_code_schema: &'a CodeSchema,
        session: &'a Session,
        args: &'a Args,
    ) -> Self {
        Migration {
            current_db_schema,
            current_code_schema,
            session,
            args,
        }
    }

    pub(crate) async fn run(&self) {
        self.run_udts().await;
        self.run_tables().await;
        self.run_materialized_views().await;

        println!("\n{}", "Migration plan ran successfully!".bright_green());
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
