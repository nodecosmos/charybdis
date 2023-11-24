mod parser;

use crate::schema::{SchemaObject, SchemaObjects};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

const MODEL_MACRO_NAME: &str = "charybdis_model";
const MATERIALIZED_VIEW_MACRO_NAME: &str = "charybdis_view_model";
const UDT_MACRO_NAME: &str = "charybdis_udt_model";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CodeSchema {
    pub tables: SchemaObjects,
    pub udts: SchemaObjects,
    pub materialized_views: SchemaObjects,
}

impl CodeSchema {
    pub fn new(project_root: &PathBuf) -> CodeSchema {
        let mut current_code_schema = CodeSchema {
            tables: SchemaObjects::new(),
            udts: SchemaObjects::new(),
            materialized_views: SchemaObjects::new(),
        };

        current_code_schema.get_models_from_code(project_root);

        current_code_schema
    }

    pub fn get_models_from_code(&mut self, project_root: &PathBuf) {
        if let Some(models_base_dir) = parser::find_src_models_dir(project_root) {
            for entry in WalkDir::new(&models_base_dir) {
                let entry: DirEntry = entry.unwrap();
                if entry.path().is_file() {
                    if entry.path().to_str().unwrap().contains("mod.rs") {
                        continue;
                    }

                    let entry_path = entry.path().to_str().unwrap().to_string();

                    if entry_path.contains("materialized_views") {
                        self.populate_materialized_views(entry);
                    } else if entry_path.contains("udts") {
                        self.populate_udts(entry);
                    } else {
                        self.populate_tables(entry);
                    }
                }
            }
        } else {
            eprintln!("Could not find 'src/models' directory.");
        }
    }

    pub fn populate_materialized_views(&mut self, entry: DirEntry) {
        let file_content: String = parser::parse_file_as_string(entry.path());
        let schema_object: SchemaObject =
            parser::parse_charybdis_model_def(&file_content, MATERIALIZED_VIEW_MACRO_NAME);
        let table_name = schema_object.table_name.clone();

        if table_name.is_empty() {
            panic!(
                "Could not find {} macro for file: {}",
                MATERIALIZED_VIEW_MACRO_NAME,
                entry.path().to_str().unwrap()
            );
        }

        self.materialized_views.insert(table_name, schema_object);
    }

    pub fn populate_udts(&mut self, entry: DirEntry) {
        let file_content: String = parser::parse_file_as_string(entry.path());
        let schema_object: SchemaObject = parser::parse_charybdis_model_def(&file_content, UDT_MACRO_NAME);
        let type_name = schema_object.type_name.clone();

        if type_name.is_empty() {
            panic!(
                "Could not find {} macro for file: {}",
                UDT_MACRO_NAME,
                entry.path().to_str().unwrap()
            );
        }

        self.udts.insert(type_name, schema_object);
    }

    pub fn populate_tables(&mut self, entry: DirEntry) {
        let file_content: String = parser::parse_file_as_string(entry.path());
        let schema_object: SchemaObject = parser::parse_charybdis_model_def(&file_content, MODEL_MACRO_NAME);
        let table_name = schema_object.table_name.clone();

        if table_name.is_empty() {
            return;
        }

        self.tables.insert(table_name, schema_object);
    }
}
