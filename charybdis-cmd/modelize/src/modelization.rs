use crate::modelization_unit::{ModelizationObjectType, ModelizationUnit};
use charybdis_parser::schema::db_schema::DbSchema;
use colored::Colorize;

pub(crate) struct Modelization<'a> {
    current_db_schema: &'a DbSchema,
}

impl<'a> Modelization<'a> {
    pub(crate) fn new(current_db_schema: &'a DbSchema) -> Self {
        Modelization { current_db_schema }
    }

    pub(crate) async fn run(&self) {
        self.run_udts().await;
        self.run_tables().await;
        self.run_materialized_views().await;

        println!("\n{}", "Modelization ran successfully!".bright_green());
    }

    async fn run_udts(&self) {
        for (name, db_udt_schema) in self.current_db_schema.udts.iter() {
            let modelization = ModelizationUnit::new(name, ModelizationObjectType::Udt, db_udt_schema);

            modelization.run().await;
        }
    }

    async fn run_tables(&self) {
        for (name, db_table_schema) in self.current_db_schema.tables.iter() {
            let modelization = ModelizationUnit::new(name, ModelizationObjectType::Udt, db_table_schema);

            modelization.run().await;
        }
    }

    async fn run_materialized_views(&self) {
        for (name, db_mv_schema) in self.current_db_schema.materialized_views.iter() {
            let modelization = ModelizationUnit::new(name, ModelizationObjectType::Udt, db_mv_schema);

            modelization.run().await;
        }
    }
}
