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


/// List all rust files under all /src dirs under the project dir.
fn _list_rust_source_files(project_root: &PathBuf) -> Vec<PathBuf> {
    // look for src/ dirs max 4 levels down, to avoid stack overflow when we hit target/
    let mut src_dirs = vec![];
    let mut it = WalkDir::new(project_root).max_depth(4).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(err)) => panic!("ERROR: {}", err),
            Some(Ok(entry)) => entry,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.ends_with("target") {
            it.skip_current_dir();
            continue;
        }
        if path.ends_with("src") {
            src_dirs.push(path.to_owned());
            it.skip_current_dir();
            continue;
        }
    }

    let mut src_files = vec![];
    for src in src_dirs {
        for entry in WalkDir::new(&src).max_depth(8) {
            let entry: DirEntry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.to_str().unwrap().ends_with(".rs") {
                src_files.push(path.to_owned());
            }
        }
    }

    src_files
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
        for entry in _list_rust_source_files(&project_root) {
            let file_content: String = parser::parse_file_as_string(&entry);
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
