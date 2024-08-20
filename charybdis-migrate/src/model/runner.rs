use crate::args::Args;
use crate::model::{ModelData, ModelType};
use colored::*;
use regex::Regex;
use scylla::Session;

pub(crate) const INDEX_SUFFIX: &str = "idx";

pub(crate) struct ModelRunner<'a> {
    session: &'a Session,
    data: &'a ModelData<'a>,
    args: &'a Args,
}

impl<'a> ModelRunner<'a> {
    pub fn new(session: &'a Session, data: &'a ModelData, args: &'a Args) -> Self {
        Self { session, data, args }
    }

    async fn execute(&self, cql: &String, print: bool) {
        if print {
            println!("{} {}", "Running CQL:".on_bright_green().black(), cql.bright_purple());
        }

        // remove all colors from cql string
        let stripped = strip_ansi_escapes::strip(cql.as_bytes());
        let cql: String = String::from_utf8(stripped).unwrap();
        let res = self.session.query(cql.clone(), ()).await;

        match res {
            Ok(_) => {
                if print {
                    println!("{}\n", "CQL executed successfully! ✅".bright_green());
                }
            }
            Err(e) => panic!("{} {}\n", "CQL execution failed! ❌".bright_red(), e),
        }
    }

    pub(crate) async fn run_first_migration(&self) {
        println!(
            "\n{} {} {}!",
            "Detected first migration for:".bright_cyan(),
            self.data.migration_object_name.bright_yellow(),
            self.data.migration_object_type.to_string().bright_yellow()
        );

        match self.data.migration_object_type {
            ModelType::Udt => {
                let cql = format!(
                    "CREATE TYPE IF NOT EXISTS {}\n(\n{}\n);\n",
                    self.data.migration_object_name,
                    self.data.current_code_schema.create_fields_clause()
                );

                self.execute(&cql, true).await;
            }
            ModelType::Table => {
                let clustering_keys = self.data.current_code_schema.clustering_keys.join(", ");
                let clustering_keys_clause = if !clustering_keys.is_empty() {
                    format!(",{}", clustering_keys)
                } else {
                    "".to_string()
                };

                let table_options = &self.data.current_code_schema.table_options;
                let mut table_options_clause = String::new();

                if let Some(table_options) = table_options {
                    table_options_clause = format!("WITH {}", table_options);
                }

                let cql = format!(
                    "CREATE TABLE IF NOT EXISTS {}\n(\n{}, \n    PRIMARY KEY (({}) {})\n) \n {}",
                    self.data.migration_object_name,
                    self.data.current_code_schema.create_fields_clause(),
                    self.data.current_code_schema.partition_keys.join(", "),
                    clustering_keys_clause,
                    table_options_clause,
                );

                self.execute(&cql, true).await;
            }
            ModelType::MaterializedView => {
                let mut primary_key = self.data.current_code_schema.partition_keys.clone();
                primary_key.append(&mut self.data.current_code_schema.clustering_keys.clone());

                let table_options = &self.data.current_code_schema.table_options;
                let mut table_options_clause = String::new();

                if let Some(table_options) = table_options {
                    table_options_clause = format!("WITH {}", table_options);
                }

                let materialized_view_where_clause = format!(
                    "WHERE {}",
                    primary_key
                        .iter()
                        .map(|field| format!("{} IS NOT NULL", field))
                        .collect::<Vec<String>>()
                        .join(" AND ")
                );

                let mv_fields_without_types = self
                    .data
                    .current_code_schema
                    .fields
                    .clone()
                    .into_iter()
                    .map(|(field_name, _, _)| field_name)
                    .collect::<Vec<String>>();

                let materialized_view_select_clause = format!(
                    "SELECT {} \nFROM {}\n{}",
                    mv_fields_without_types.join(", "),
                    self.data.current_code_schema.base_table.clone(),
                    materialized_view_where_clause
                );

                let primary_key_clause = format!(
                    "PRIMARY KEY (({}), {})\n",
                    self.data.current_code_schema.partition_keys.join(", "),
                    self.data.current_code_schema.clustering_keys.join(", ")
                );

                let cql = format!(
                    "CREATE MATERIALIZED VIEW IF NOT EXISTS {}\nAS {}\n{}\n{}",
                    self.data.migration_object_name,
                    materialized_view_select_clause,
                    primary_key_clause,
                    table_options_clause
                );

                self.execute(&cql, true).await;
            }
        }
    }

    pub(crate) async fn run_field_added_migration(&self) {
        println!(
            "\n{} {} {}",
            "Detected new fields in".bright_cyan(),
            self.data.migration_object_name.bright_blue(),
            self.data.migration_object_type.to_string().bright_yellow()
        );

        if self.data.migration_object_type == ModelType::Table {
            self.run_table_field_added_migration().await;
        } else {
            self.run_udt_field_added_migration().await;
        }
    }

    async fn run_table_field_added_migration(&self) {
        let add_fields_clause = self
            .data
            .new_fields
            .iter()
            .map(|(field_name, field_type)| format!("{} {}", field_name, field_type))
            .collect::<Vec<String>>()
            .join(", ");

        let cql = format!(
            "ALTER {} {} ADD ({})",
            self.data.migration_object_type, self.data.migration_object_name, add_fields_clause,
        );

        self.execute(&cql, true).await;
    }

    async fn run_udt_field_added_migration(&self) {
        for (field_name, field_type) in self.data.new_fields.iter() {
            let cql = format!(
                "ALTER TYPE {} ADD {} {}",
                self.data.migration_object_name, field_name, field_type
            );

            self.execute(&cql, true).await;
        }
    }

    pub(crate) async fn run_field_removed_migration(&self) {
        println!(
            "\n{} {} {}",
            "Detected removed fields in".bright_cyan(),
            self.data.migration_object_name.bright_yellow(),
            self.data.migration_object_type.to_string().bright_yellow()
        );

        let removed_fields = self.data.removed_fields.join(", ");

        let cql = format!(
            "ALTER {} {} DROP ({})",
            self.data.migration_object_type, self.data.migration_object_name, removed_fields,
        );

        self.execute(&cql, true).await;
    }

    pub(crate) async fn run_field_type_changed_migration(&self) {
        println!(
            "{}",
            "Field Type Change Migration (Drop and replace):"
                .on_bright_green()
                .black()
        );

        // remove fields with changed types
        let changed_fields = self
            .data
            .changed_field_types
            .iter()
            .map(|(field_name, _, _)| field_name.clone())
            .collect::<Vec<String>>()
            .join(", ");

        let cql = format!(
            "ALTER {} {} DROP ({})",
            self.data.migration_object_type, self.data.migration_object_name, changed_fields,
        );

        self.execute(&cql, true).await;

        let add_fields_clause = self
            .data
            .changed_field_types
            .iter()
            .map(|(field_name, _, field_type)| format!("{} {}", field_name, field_type))
            .collect::<Vec<String>>()
            .join(", ");

        let cql = format!(
            "ALTER {} {} ADD ({})",
            self.data.migration_object_type, self.data.migration_object_name, add_fields_clause,
        );

        self.execute(&cql, true).await;
    }

    pub(crate) async fn run_global_index_added_migration(&self) {
        println!(
            "\n{} {} {}",
            "Detected new indexes in ".bright_cyan(),
            self.data.migration_object_name.bright_yellow(),
            self.data.migration_object_type.to_string().bright_yellow()
        );

        for column_name in &self.data.new_global_secondary_indexes {
            let index_name: String = self.data.construct_index_name(column_name);

            let cql = format!(
                "CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                index_name, self.data.migration_object_name, column_name,
            );

            self.execute(&cql, true).await;
        }
    }

    pub(crate) async fn run_global_index_removed_migration(&self) {
        println!(
            "\n{} {} {}",
            "Detected removed indexes for ".bright_cyan(),
            self.data.migration_object_name.bright_yellow(),
            self.data.migration_object_type.to_string().bright_yellow()
        );

        for index in &self.data.removed_global_secondary_indexes {
            let cql = format!("DROP INDEX {}", index,);

            self.execute(&cql, true).await;
        }
    }

    pub(crate) async fn run_local_index_added_migration(&self) {
        println!(
            "\n{} {} {}",
            "Detected new local indexes in ".bright_cyan(),
            self.data.migration_object_name.bright_yellow(),
            self.data.migration_object_type.to_string().bright_yellow()
        );

        for local_secondary_index in &self.data.new_local_secondary_indexes {
            let partition_keys = self.data.current_code_schema.partition_keys.clone();

            let mut idx_name = partition_keys.join("_");
            idx_name.push('_');
            idx_name.push_str(local_secondary_index);

            let index_name: String = self.data.construct_index_name(&idx_name);
            let pks = partition_keys.join(", ");
            let cql = format!(
                "CREATE INDEX IF NOT EXISTS {} ON {} (({}), {})",
                index_name, self.data.migration_object_name, pks, local_secondary_index,
            );

            self.execute(&cql, true).await;
        }
    }

    pub(crate) async fn run_local_index_removed_migration(&self) {
        println!(
            "\n{} {} {}",
            "Detected removed local indexes for ".bright_cyan(),
            self.data.migration_object_name.bright_yellow(),
            self.data.migration_object_type.to_string().bright_yellow()
        );

        for index in &self.data.removed_local_secondary_indexes {
            let cql = format!("DROP INDEX {}", index,);

            self.execute(&cql, true).await;
        }
    }

    pub(crate) async fn run_table_options_change_migration(&self) {
        if self.data.migration_object_type == ModelType::Table
            || self.data.migration_object_type == ModelType::MaterializedView
        {
            if let Some(alter_table_options) = self.extract_alter_table_options() {
                let cql = format!(
                    "\n ALTER TABLE {} WITH {}",
                    self.data.migration_object_name, alter_table_options
                );

                self.execute(&cql, self.args.verbose).await;
            }
        }
    }

    fn extract_alter_table_options(&self) -> Option<String> {
        // strip clustering order and compact storage options from table options
        // because they are not supported by ALTER TABLE
        if let Some(table_options) = &self.data.current_code_schema.table_options {
            let table_options = table_options.replace("WITH", "").trim().to_string();
            let compact_storage_re = Regex::new(r"(?i)\bCOMPACT STORAGE\b\s*(AND\s*)?").unwrap();
            let clustering_order_re = Regex::new(r"(?i)\bCLUSTERING ORDER BY\b[^)]+\)\s*(AND\s*)?").unwrap();

            let stripped_co_string = compact_storage_re.replace_all(table_options.as_str(), "");
            let alter_table_options = clustering_order_re.replace_all(&stripped_co_string, "").to_string();

            if alter_table_options.is_empty() {
                return None;
            }

            Some(alter_table_options.to_string())
        } else {
            None
        }
    }
}
