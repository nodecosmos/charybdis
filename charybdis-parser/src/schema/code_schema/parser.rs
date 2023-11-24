use crate::fields::CharybdisFields;
use crate::macro_args::CharybdisMacroArgs;
use crate::schema::SchemaObject;
use colored::Colorize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use syn::{Field, Fields, GenericArgument, Item, PathArguments};
use walkdir::WalkDir;

// returns models dir if nested in src/models
pub(crate) fn find_src_models_dir(project_root: &PathBuf) -> Option<PathBuf> {
    for entry in WalkDir::new(project_root) {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() && entry.file_name().to_string_lossy() == "models" {
            let parent_dir = entry.path().parent()?;
            if parent_dir.file_name().unwrap().to_string_lossy() == "src" {
                println!("{}\n", "Detected 'src/models' directory".bright_green().bold());
                return Some(entry.into_path());
            }
        }
    }
    None
}

pub(crate) fn parse_file_as_string(path: &Path) -> String {
    let mut file_content = String::new();
    File::open(path).unwrap().read_to_string(&mut file_content).unwrap();
    file_content
}

pub(crate) fn parse_charybdis_model_def(file_content: &str, macro_name: &str) -> SchemaObject {
    let ast: syn::File = syn::parse_file(file_content)
        .map_err(|e| {
            println!(
                "{}\n",
                format!("Error parsing file: {}", file_content).bright_red().bold()
            );
            e
        })
        .unwrap();

    let mut schema_object: SchemaObject = SchemaObject::new();

    for item in ast.items {
        if let Item::Struct(item_struct) = item {
            let is_charybdis_model = item_struct.attrs.iter().any(|attr| attr.path().is_ident(macro_name));

            // If the struct doesn't have the required macro, continue to the next item
            if !is_charybdis_model {
                continue;
            }

            if let Fields::Named(fields_named) = item_struct.fields {
                let fields = CharybdisFields::new(&fields_named);
                for field in fields.db_fields() {
                    if let Field {
                        ident: Some(ident),
                        ty: syn::Type::Path(type_path),
                        ..
                    } = field
                    {
                        let field_name = ident.to_string();
                        let field_type = type_with_arguments(&type_path);

                        schema_object.fields.insert(field_name, field_type);
                    }
                }
            }

            // parse charybdis macro content
            for attr in &item_struct.attrs {
                if attr.path().is_ident(macro_name) {
                    let args: CharybdisMacroArgs = attr.parse_args().unwrap();

                    schema_object.table_name = args.table_name.unwrap_or("".to_string());
                    schema_object.type_name = args.type_name.unwrap_or("".to_string());
                    schema_object.base_table = args.base_table.unwrap_or("".to_string());

                    schema_object.partition_keys = args.partition_keys.unwrap_or(vec![]);
                    schema_object.clustering_keys = args.clustering_keys.unwrap_or(vec![]);

                    if let Some(gsi) = args.global_secondary_indexes {
                        gsi.iter().for_each(|global_idx| {
                            schema_object
                                .global_secondary_indexes
                                .push(("".to_string(), global_idx.to_string()));
                        });
                    }

                    if let Some(lsi) = args.local_secondary_indexes {
                        lsi.iter().for_each(|local_idx| {
                            schema_object
                                .local_secondary_indexes
                                .push(("".to_string(), local_idx.clone()));
                        });
                    }

                    schema_object.table_options = args.table_options;
                }
            }
        }
    }

    schema_object
}

fn type_with_arguments(type_path: &syn::TypePath) -> String {
    let first_segment = &type_path.path.segments[0];
    let mut type_name = quote::quote! { #type_path }.to_string();

    // Check if the type is an Option<T>
    if first_segment.ident == "Option" {
        if let PathArguments::AngleBracketed(angle_bracketed_args) = &first_segment.arguments {
            if let Some(GenericArgument::Type(inner_type)) = angle_bracketed_args.args.first() {
                // Return the inner type of Option<T>
                type_name = quote::quote! { #inner_type }.to_string();
            }
        }
    }

    // strip if full path is provided
    if type_name.contains("::") {
        type_name = type_name.split("::").last().unwrap().to_string();
    }

    type_name
}
