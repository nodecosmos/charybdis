pub mod code_schema;
pub mod db_schema;
pub mod secondary_indexes;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
pub type FieldName = String;
pub type FieldType = String;
pub type IsStatic = bool;
pub type IndexName = String;
pub type IdxField = String;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SchemaObject {
    pub fields: Vec<(FieldName, FieldType, IsStatic)>,
    pub field_names: HashSet<String>,
    pub types_by_name: HashMap<String, String>,
    pub type_name: String,
    pub table_name: String,
    pub base_table: String,
    pub partition_keys: Vec<String>,
    pub clustering_keys: Vec<String>,
    pub static_columns: Vec<String>,
    pub global_secondary_indexes: Vec<(IndexName, IdxField)>,
    pub local_secondary_indexes: Vec<(IndexName, IdxField)>,
    pub table_options: Option<String>,
}

impl SchemaObject {
    pub(crate) fn push_field(&mut self, field_name: String, field_type: String, is_static: bool) {
        self.fields.push((field_name.clone(), field_type.clone(), is_static));
        self.field_names.insert(field_name.clone());
        self.types_by_name.insert(field_name, field_type);
    }

    pub fn contains_field(&self, field_name: &str) -> bool {
        self.field_names.contains(field_name)
    }
}

impl SchemaObject {
    pub(crate) fn new() -> Self {
        SchemaObject {
            fields: Vec::new(),
            field_names: HashSet::new(),
            types_by_name: HashMap::new(),
            type_name: String::new(),
            table_name: String::new(),
            base_table: String::new(),
            partition_keys: Vec::new(),
            clustering_keys: Vec::new(),
            static_columns: Vec::new(),
            global_secondary_indexes: Vec::new(),
            local_secondary_indexes: Vec::new(),
            table_options: None,
        }
    }

    pub fn create_fields_clause(&self) -> String {
        let mut cql_fields = String::new();

        for (field_name, field_type, is_static) in self.fields.iter() {
            cql_fields.push_str(&format!(
                "    {} {} {},\n",
                field_name.bright_cyan().bold(),
                field_type.bright_yellow(),
                if *is_static { "static" } else { "" }
            ));
        }

        cql_fields.pop();
        cql_fields.pop();
        cql_fields
    }
}

pub type ModelName = String;
pub type SchemaObjects = HashMap<ModelName, SchemaObject>;
