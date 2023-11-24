use crate::migration_unit::data::MigrationUnitData;
use crate::migration_unit::{MigrationObjectType, MigrationUnit};
use charybdis_parser::schema::code_schema::CodeSchema;
use charybdis_parser::schema::db_schema::DbSchema;
use charybdis_parser::schema::SchemaObject;
use colored::Colorize;
use scylla::Session;

pub(crate) struct Migration<'a> {
    current_db_schema: &'a DbSchema,
    current_code_schema: &'a CodeSchema,
    session: &'a Session,
}

impl<'a> Migration<'a> {
    pub(crate) fn new(
        current_db_schema: &'a DbSchema,
        current_code_schema: &'a CodeSchema,
        session: &'a Session,
    ) -> Self {
        Migration {
            current_db_schema,
            current_code_schema,
            session,
        }
    }

    pub(crate) async fn run(&self) {
        self.run_udts().await;
        self.run_tables().await;
        self.run_materialized_views().await;

        println!("\n{}", "Migration plan ran successfully!".bright_green());
    }

    async fn run_udts(&self) {
        let empty_udt = SchemaObject::new();

        for (name, code_udt_schema) in self.current_code_schema.udts.iter() {
            let migration_unit_data = MigrationUnitData::new(
                name,
                MigrationObjectType::Udt,
                code_udt_schema,
                self.current_db_schema.udts.get(name).unwrap_or(&empty_udt),
            );

            let migration = MigrationUnit::new(&migration_unit_data, self.session);

            migration.run().await;
        }
    }

    async fn run_tables(&self) {
        let empty_table = SchemaObject::new();

        for (name, code_table_schema) in self.current_code_schema.tables.iter() {
            let migration_unit_data = MigrationUnitData::new(
                name,
                MigrationObjectType::Table,
                code_table_schema,
                self.current_db_schema.tables.get(name).unwrap_or(&empty_table),
            );

            let migration = MigrationUnit::new(&migration_unit_data, self.session);

            migration.run().await;
        }
    }

    async fn run_materialized_views(&self) {
        let empty_mv = SchemaObject::new();

        for (name, code_mv_schema) in self.current_code_schema.materialized_views.iter() {
            let migration_unit_data = MigrationUnitData::new(
                name,
                MigrationObjectType::MaterializedView,
                code_mv_schema,
                self.current_db_schema.materialized_views.get(name).unwrap_or(&empty_mv),
            );

            let migration = MigrationUnit::new(&migration_unit_data, self.session);

            migration.run().await;
        }
    }
}
