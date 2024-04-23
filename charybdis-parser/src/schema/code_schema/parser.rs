use std::fs::File;
use std::io::Read;
use std::path::Path;

use syn::{Fields, GenericArgument, Item, ItemStruct, PathArguments};

use crate::fields::CharybdisFields;
use crate::schema::code_schema::ModelMacro;
use crate::schema::SchemaObject;
use crate::traits::CharybdisMacroArgs;

pub(crate) fn parse_file_as_string(path: &Path) -> String {
    let mut file_content = String::new();

    File::open(path)
        .unwrap()
        .read_to_string(&mut file_content)
        .expect(format!("Unable to read file: {}", path.display()).as_str());

    file_content
}

pub(crate) fn parse_charybdis_model_def(ast: &syn::File, model_macro: ModelMacro) -> Vec<SchemaObject> {
    let mut schema_objects: Vec<SchemaObject> = Vec::new();

    for item in &ast.items {
        if let Item::Struct(item_struct) = item {
            // If the struct doesn't have the required macro, continue to the next item.
            if !item_struct
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident(model_macro.to_string().as_str()))
            {
                continue;
            }

            schema_objects.push(extract_schema_object(&item_struct, &model_macro));
        }
    }

    schema_objects
}

fn extract_schema_object(item_struct: &ItemStruct, model_macro: &ModelMacro) -> SchemaObject {
    let mut schema_object: SchemaObject = SchemaObject::new();

    for attr in &item_struct.attrs {
        if attr.path().is_ident(model_macro.to_string().as_str()) {
            let args: CharybdisMacroArgs = attr.parse_args().unwrap();

            if let Some(table_name) = args.table_name {
                schema_object.table_name = table_name;
            } else if model_macro == &ModelMacro::Table {
                panic!("Table name is required in charybdis_model macro");
            }

            if let Some(base_table) = args.base_table {
                schema_object.base_table = base_table;
            } else if model_macro == &ModelMacro::MaterializedView {
                panic!("Base table is required in charybdis_view_model macro");
            }

            if let Some(type_name) = args.type_name {
                schema_object.type_name = type_name;
            } else if model_macro == &ModelMacro::Udt {
                panic!("Type name is required in charybdis_udt_model macro");
            }

            if let Some(partition_keys) = args.partition_keys {
                schema_object.partition_keys = partition_keys;
            } else if model_macro == &ModelMacro::Table {
                panic!("Partition keys are required in charybdis_model macro");
            }

            schema_object.clustering_keys = args.clustering_keys.unwrap_or_default();
            schema_object.static_columns = args.static_columns.unwrap_or_default();

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

    // parse struct fields
    if let Fields::Named(fields_named) = &item_struct.fields {
        let db_fields = CharybdisFields::db_fields(&fields_named);

        for field in db_fields {
            let field_name = field.ident.to_string();
            let field_type = type_with_arguments(&field.ty_path);
            let is_static = schema_object.static_columns.contains(&field_name);

            schema_object.push_field(field_name, field_type, is_static);
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
