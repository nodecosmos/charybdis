use std::fmt::Display;
use std::path::PathBuf;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::schema::{SchemaObject, SchemaObjects};

mod parser;

#[derive(Eq, PartialEq)]
pub(crate) enum ModelMacro {
    Table,
    Udt,
    MaterializedView,
}

impl Display for ModelMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelMacro::Table => write!(f, "charybdis_model"),
            ModelMacro::MaterializedView => write!(f, "charybdis_view_model"),
            ModelMacro::Udt => write!(f, "charybdis_udt_model"),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CodeSchema {
    pub tables: SchemaObjects,
    pub udts: SchemaObjects,
    pub materialized_views: SchemaObjects,
}

impl CodeSchema {
    pub fn new(project_root: &String) -> CodeSchema {
        let mut current_code_schema = CodeSchema {
            tables: SchemaObjects::new(),
            udts: SchemaObjects::new(),
            materialized_views: SchemaObjects::new(),
        };

        current_code_schema.get_models_from_code(project_root);

        current_code_schema
    }

    pub fn get_models_from_code(&mut self, project_root: &String) {
        let project_root: PathBuf = PathBuf::from(project_root);
        for entry in WalkDir::new(project_root) {
            let entry: DirEntry = entry.unwrap();

            if entry.path().is_file() {
                let path = entry.path().to_str().unwrap();

                if !path.ends_with(".rs") {
                    continue;
                }

                let file_content: String = parser::parse_file_as_string(entry.path());
                let ast: syn::File = syn::parse_file(&file_content)
                    .map_err(|e| {
                        println!(
                            "{}\n",
                            format!("Error parsing file: {}", file_content).bright_red().bold()
                        );
                        e
                    })
                    .unwrap();

                self.populate_materialized_views(&ast);
                self.populate_udts(&ast);
                self.populate_tables(&ast);
            }
        }
    }

    pub fn populate_materialized_views(&mut self, ast: &syn::File) {
        let schema_objects: Vec<SchemaObject> = parser::parse_charybdis_model_def(ast, ModelMacro::MaterializedView);

        schema_objects.into_iter().for_each(|schema_object| {
            let table_name = schema_object.table_name.clone();

            self.materialized_views.insert(table_name, schema_object);
        });
    }

    pub fn populate_udts(&mut self, ast: &syn::File) {
        let schema_objects: Vec<SchemaObject> = parser::parse_charybdis_model_def(ast, ModelMacro::Udt);

        schema_objects.into_iter().for_each(|schema_object| {
            let type_name = schema_object.type_name.to_lowercase().clone();

            self.udts.insert(type_name, schema_object);
        });
    }

    pub fn populate_tables(&mut self, ast: &syn::File) {
        let schema_object: Vec<SchemaObject> = parser::parse_charybdis_model_def(ast, ModelMacro::Table);

        schema_object.into_iter().for_each(|schema_object| {
            let table_name = schema_object.table_name.clone();

            self.tables.insert(table_name, schema_object);
        });
    }
}
