pub(crate) mod data;
mod runner;

use crate::model::data::ModelData;
use crate::model::runner::ModelRunner;
use crate::Args;
use colored::Colorize;
use scylla::Session;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum ModelType {
    Udt,
    Table,
    MaterializedView,
}

impl Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Udt => write!(f, "UDT"),
            ModelType::Table => write!(f, "Table"),
            ModelType::MaterializedView => write!(f, "Materialized View"),
        }
    }
}

/// Migration steps in non-conflicting order
enum MigrationStep {
    ChangeTableOptions,
    ChangeFieldTypes,
    AddFields,
    AddGlobalIndexes,
    AddLocalIndexes,
    RemoveLocalIndexes,
    RemoveGlobalIndexes,
    RemoveFields,
}

impl MigrationStep {
    fn array() -> [MigrationStep; 8] {
        [
            MigrationStep::ChangeTableOptions,
            MigrationStep::ChangeFieldTypes,
            MigrationStep::AddFields,
            MigrationStep::AddGlobalIndexes,
            MigrationStep::AddLocalIndexes,
            MigrationStep::RemoveLocalIndexes,
            MigrationStep::RemoveGlobalIndexes,
            MigrationStep::RemoveFields,
        ]
    }
}

pub(crate) struct ModelMigration<'a> {
    data: &'a ModelData<'a>,
    runner: ModelRunner<'a>,
    args: &'a Args,
}

impl<'a> ModelMigration<'a> {
    pub(crate) fn new(data: &'a ModelData, session: &'a Session, args: &'a Args) -> Self {
        let runner = ModelRunner::new(&session, data, args);

        Self { data, runner, args }
    }

    pub(crate) async fn run(&self) {
        if self.data.is_first_migration() {
            self.handle_first_migration().await;
            return;
        }

        self.panic_on_partition_key_change();
        self.panic_on_clustering_key_change();

        let mut is_any_field_changed = false;

        for step in MigrationStep::array().iter() {
            match step {
                MigrationStep::ChangeTableOptions => self.runner.run_table_options_change_migration().await,
                MigrationStep::ChangeFieldTypes => {
                    if self.data.has_changed_type_fields() {
                        is_any_field_changed = true;
                        self.handle_fields_type_change().await;
                    }
                }
                MigrationStep::AddFields => {
                    if self.data.has_new_fields() {
                        is_any_field_changed = true;
                        self.handle_new_fields().await;
                    }
                }
                MigrationStep::AddGlobalIndexes => {
                    if self.data.has_new_global_secondary_indexes() {
                        is_any_field_changed = true;
                        self.runner.run_global_index_added_migration().await;
                    }
                }
                MigrationStep::AddLocalIndexes => {
                    if self.data.has_new_local_secondary_indexes() {
                        is_any_field_changed = true;
                        self.runner.run_local_index_added_migration().await;
                    }
                }
                MigrationStep::RemoveGlobalIndexes => {
                    if self.data.has_removed_global_secondary_indexes() {
                        is_any_field_changed = true;
                        self.runner.run_global_index_removed_migration().await;
                    }
                }
                MigrationStep::RemoveLocalIndexes => {
                    if self.data.has_removed_local_secondary_indexes() {
                        is_any_field_changed = true;
                        self.runner.run_local_index_removed_migration().await;
                    }
                }
                MigrationStep::RemoveFields => {
                    if self.data.has_removed_fields() {
                        is_any_field_changed = true;
                        self.handle_removed_fields().await;
                    }
                }
            }
        }

        if !is_any_field_changed {
            println!(
                "{} {} {}",
                "No field changes in".green(),
                self.data.migration_object_name.bright_yellow(),
                self.data.migration_object_type.to_string().bright_magenta()
            );
        }
    }

    async fn handle_first_migration(&self) {
        self.runner.run_first_migration().await;

        if self.data.has_new_global_secondary_indexes() {
            self.runner.run_global_index_added_migration().await;
        }

        if self.data.has_new_local_secondary_indexes() {
            self.runner.run_local_index_added_migration().await;
        }
    }

    async fn handle_fields_type_change(&self) {
        if self.args.drop_and_replace {
            self.panic_on_mv_fields_change();
            self.panic_on_udt_fields_removal();

            self.runner.run_field_type_changed_migration().await;
        } else {
            self.panic_on_field_type_change();
        }
    }

    async fn handle_new_fields(&self) {
        self.panic_on_mv_fields_change();

        self.runner.run_field_added_migration().await;
    }

    async fn handle_removed_fields(&self) {
        self.panic_on_mv_fields_change();
        self.panic_on_udt_fields_removal();

        self.runner.run_field_removed_migration().await;
    }

    fn panic_on_field_type_change(&self) {
        panic!(
            "\n\n{} {} {}\n{}\n\n",
            "Illegal change in".bright_red(),
            self.data.migration_object_name.bright_yellow(),
            self.data.migration_object_type.to_string().bright_magenta(),
            "Field type change is not allowed. Use `-d` flag to drop and recreate field with new type!".bright_red(),
        );
    }

    fn panic_on_partition_key_change(&self) {
        if self.data.migration_object_type != ModelType::Udt {
            if self.data.partition_key_changed() {
                panic!(
                    "\n\n{} {} {}\n{}\n\n",
                    "Illegal change in".bright_red(),
                    self.data.migration_object_name.bright_yellow(),
                    self.data.migration_object_type.to_string().bright_magenta(),
                    "Partition key change is not allowed!".bright_red(),
                );
            }
        }
    }

    fn panic_on_clustering_key_change(&self) {
        if self.data.migration_object_type != ModelType::Udt {
            if self.data.clustering_key_changed() {
                panic!(
                    "\n\n{} {} {}\n{}\n\n",
                    "Illegal change in".bright_red(),
                    self.data.migration_object_name.bright_yellow(),
                    self.data.migration_object_type.to_string().bright_magenta(),
                    "Clustering key change is not allowed!".bright_red(),
                );
            }
        }
    }

    fn panic_on_udt_fields_removal(&self) {
        if self.data.migration_object_type == ModelType::Udt
            && (self.data.has_removed_fields() || self.data.has_changed_type_fields())
        {
            panic!("\n{}\n", "UDT fields removal is not allowed!".bold().bright_red());
        }
    }

    fn panic_on_mv_fields_change(&self) {
        if self.data.migration_object_type == ModelType::MaterializedView {
            panic!(
                "\n{}\n",
                "Materialized view fields change is not allowed!".bold().bright_red()
            );
        }
    }
}
