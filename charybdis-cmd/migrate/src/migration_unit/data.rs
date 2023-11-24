use crate::migration_unit::runner::INDEX_SUFFIX;
use crate::migration_unit::MigrationObjectType;
use charybdis_parser::schema::secondary_indexes::LocalIndexTarget;
use charybdis_parser::schema::{IndexName, SchemaObject};
use colored::Colorize;

pub type FieldName = String;
pub type FieldType = String;
type NewField = (FieldName, FieldType);

pub struct MigrationUnitData<'a> {
    pub(crate) migration_object_name: &'a String,
    pub(crate) migration_object_type: MigrationObjectType,
    pub(crate) current_code_schema: &'a SchemaObject,
    pub(crate) current_db_schema: &'a SchemaObject,
    pub(crate) new_fields: Vec<NewField>,
    pub(crate) removed_fields: Vec<FieldName>,
    pub(crate) new_global_secondary_indexes: Vec<FieldName>,
    pub(crate) new_local_secondary_indexes: Vec<LocalIndexTarget>,
    pub(crate) removed_global_secondary_indexes: Vec<IndexName>,
    pub(crate) removed_local_secondary_indexes: Vec<IndexName>,
}

impl<'a> MigrationUnitData<'a> {
    pub(crate) fn new(
        migration_object_name: &'a String,
        migration_object_type: MigrationObjectType,
        current_code_schema: &'a SchemaObject,
        current_db_schema: &'a SchemaObject,
    ) -> Self {
        let mut data = Self {
            migration_object_name,
            migration_object_type,
            current_code_schema,
            current_db_schema,
            new_fields: vec![],
            removed_fields: vec![],
            new_global_secondary_indexes: vec![],
            new_local_secondary_indexes: vec![],
            removed_global_secondary_indexes: vec![],
            removed_local_secondary_indexes: vec![],
        };

        data.fetch_new_fields();
        data.fetch_removed_fields();
        data.fetch_new_global_secondary_indexes();
        data.fetch_removed_global_secondary_indexes();
        data.fetch_new_local_secondary_indexes();
        data.fetch_removed_local_secondary_indexes();

        data
    }

    pub(crate) fn construct_index_name(&self, column_name: &String) -> String {
        format!("{}_{}_{}", self.migration_object_name, column_name, INDEX_SUFFIX)
    }

    pub(crate) fn is_first_migration(&self) -> bool {
        self.current_db_schema.fields.is_empty()
    }

    pub(crate) fn has_new_global_secondary_indexes(&self) -> bool {
        !self.new_global_secondary_indexes.is_empty()
    }

    pub(crate) fn has_removed_global_secondary_indexes(&self) -> bool {
        !self.removed_global_secondary_indexes.is_empty()
    }

    pub(crate) fn has_new_local_secondary_indexes(&self) -> bool {
        !self.new_local_secondary_indexes.is_empty()
    }

    pub(crate) fn has_removed_local_secondary_indexes(&self) -> bool {
        !self.removed_local_secondary_indexes.is_empty()
    }

    pub(crate) fn has_new_fields(&self) -> bool {
        !self.new_fields.is_empty()
    }

    pub(crate) fn has_removed_fields(&self) -> bool {
        !self.removed_fields.is_empty()
    }

    // Checks if any field of db schema has changed type in code schema.
    pub(crate) fn field_type_changed(&self) -> bool {
        for (field_name, field_type) in self.current_code_schema.fields.iter() {
            if let Some(db_field_type) = self.current_db_schema.fields.get(field_name) {
                let code_field_type = field_type.to_lowercase().replace(' ', "");
                let db_field_type = db_field_type.to_lowercase().replace(' ', "");

                if code_field_type != db_field_type {
                    println!(
                        "\nType Change: {} -> {}",
                        db_field_type.to_uppercase().yellow().bold(),
                        code_field_type.to_uppercase().bright_red().bold()
                    );

                    return true;
                }
            }
        }

        false
    }

    pub(crate) fn partition_key_changed(&self) -> bool {
        let mut code_partition_keys: Vec<String> = self.current_code_schema.partition_keys.clone();
        let mut db_partition_keys = self.current_db_schema.partition_keys.clone();

        code_partition_keys.sort();
        db_partition_keys.sort();

        code_partition_keys != db_partition_keys
    }

    pub(crate) fn clustering_key_changed(&self) -> bool {
        let mut code_clustering_keys = self.current_code_schema.clustering_keys.clone();
        let mut db_clustering_keys = self.current_db_schema.clustering_keys.clone();

        code_clustering_keys.sort();
        db_clustering_keys.sort();

        code_clustering_keys != db_clustering_keys
    }

    fn fetch_new_fields(&mut self) {
        for (field_name, field_type) in self.current_code_schema.fields.iter() {
            if !self.current_db_schema.fields.contains_key(field_name) {
                self.new_fields.push((field_name.clone(), field_type.clone()));
            }
        }
    }

    fn fetch_removed_fields(&mut self) {
        for (field_name, _) in self.current_db_schema.fields.iter() {
            if !self.current_code_schema.fields.contains_key(field_name) {
                self.removed_fields.push(field_name.clone());
            }
        }
    }

    fn fetch_new_global_secondary_indexes(&mut self) {
        let _ = &self
            .current_code_schema
            .global_secondary_indexes
            .iter()
            .for_each(|(_index_name, index_target)| {
                if !self
                    .current_db_schema
                    .global_secondary_indexes
                    .iter()
                    .any(|(_, target)| target == index_target)
                {
                    self.new_global_secondary_indexes.push(index_target.clone());
                }
            });
    }

    fn fetch_removed_global_secondary_indexes(&mut self) {
        let _ = &self
            .current_db_schema
            .global_secondary_indexes
            .iter()
            .for_each(|(index_name, index_target)| {
                if !self
                    .current_code_schema
                    .global_secondary_indexes
                    .iter()
                    .any(|(_index_name, target)| target == index_target)
                {
                    self.removed_global_secondary_indexes.push(index_name.clone());
                }
            });
    }

    fn fetch_new_local_secondary_indexes(&mut self) {
        let _ = &self
            .current_code_schema
            .local_secondary_indexes
            .iter()
            .for_each(|(_index_name, index_target)| {
                if !self
                    .current_db_schema
                    .local_secondary_indexes
                    .iter()
                    .any(|(_, target)| target == index_target)
                {
                    self.new_local_secondary_indexes.push(index_target.clone());
                }
            });
    }

    fn fetch_removed_local_secondary_indexes(&mut self) {
        let _ = &self
            .current_db_schema
            .local_secondary_indexes
            .iter()
            .for_each(|(index_name, index_target)| {
                if !self
                    .current_code_schema
                    .local_secondary_indexes
                    .iter()
                    .any(|(_index_name, target)| target == index_target)
                {
                    self.removed_local_secondary_indexes.push(index_name.clone());
                }
            });
    }
}
