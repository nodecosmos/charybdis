extern crate proc_macro;
mod char_model_impls;
mod macro_rules;
mod native;
mod scylla_impls;
mod utils;

use crate::char_model_impls::*;
use crate::macro_rules::*;
use crate::native::{
    delete_by_primary_key_functions, find_by_primary_keys_functions, pull_from_collection_fields_query_consts,
    pull_from_collection_funs, push_to_collection_fields_query_consts, push_to_collection_funs,
};
use crate::scylla_impls::{from_row, serialized};
use charybdis_parser::fields::{strip_charybdis_attributes, CharybdisFields};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

/// This macro generates the implementation of the [Model] trait for the given struct.
#[proc_macro_attribute]
pub fn charybdis_model(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: CharybdisMacroArgs = parse_macro_input!(args);
    let mut input: DeriveInput = parse_macro_input!(input);
    let char_fields = CharybdisFields::from_input(&input);
    let db_fields = char_fields.db_fields();
    let all_fields = char_fields.all_fields();

    strip_charybdis_attributes(&mut input);

    let struct_name = &input.ident;

    // Charybdis::Model consts
    let db_model_name_const = db_model_name_const(&args);
    let partition_keys_const = partition_keys_const(&args);
    let clustering_keys_const = clustering_keys_const(&args);
    let primary_key_const = primary_key_const(&args);
    let select_fields_clause = select_fields_clause(&args, &db_fields);
    let find_by_primary_key_query_const = find_by_primary_key_query_const(&args, &db_fields);
    let find_by_partition_key_query_const = find_by_partition_key_query_const(&args, &db_fields);
    let insert_query_const = insert_query_const(&args, &db_fields);
    let update_query_const = update_query_const(&args, &db_fields);
    let delete_query_const = delete_query_const(&args);
    let delete_by_partition_key_query_const = delete_by_partition_key_query_const(&args);

    // Charybdis::Model methods
    let primary_key_values = primary_key_values(&args);
    let partition_key_values = partition_key_values(&args);
    let clustering_key_values = clustering_key_values(&args);
    let update_values = update_values(&args, &db_fields);

    // Collection consts
    let push_to_collection_fields_query_consts = push_to_collection_fields_query_consts(&args, &db_fields);
    let pull_from_collection_fields_query_consts = pull_from_collection_fields_query_consts(&args, &db_fields);

    // Collection methods
    let push_to_collection_funs = push_to_collection_funs(&args, &db_fields);
    let pull_from_collection_funs = pull_from_collection_funs(&args, &db_fields);

    // ValueList trait
    let serialized = serialized(&db_fields, &all_fields);

    // FromRow trait
    let from_row = from_row(struct_name, &db_fields, &all_fields);

    // Current model rules
    let find_model_query_rule = find_model_query_rule(&args, &db_fields, struct_name);
    let find_model_rule = find_model_rule(&args, &db_fields, struct_name);
    let find_first_model_rule = find_first_model_rule(&args, &db_fields, struct_name);
    let update_model_query_rule = update_model_query_rule(&args, struct_name);

    // Associated functions for finding by primary key
    let find_by_key_funs = find_by_primary_keys_functions(&args, &db_fields, struct_name);
    let delete_by_cks_funs = delete_by_primary_key_functions(&args, &db_fields, struct_name);

    let partial_model_generator = partial_model_macro_generator(args, &input);

    let expanded = quote! {
        #input

        impl #struct_name {
            #find_by_key_funs
            #delete_by_cks_funs
            #push_to_collection_fields_query_consts
            #pull_from_collection_fields_query_consts
            // methods
            #push_to_collection_funs
            #pull_from_collection_funs
        }

       impl charybdis::model::BaseModel for #struct_name {
            // consts
            #db_model_name_const
            #clustering_keys_const
            #partition_keys_const
            #primary_key_const
            #find_by_primary_key_query_const
            #find_by_partition_key_query_const
            #select_fields_clause
            // methods
            #primary_key_values
            #partition_key_values
            #clustering_key_values
        }

        impl charybdis::model::Model for #struct_name {
            // operation consts
            #insert_query_const
            #update_query_const
            #delete_query_const
            #delete_by_partition_key_query_const
            // methods
            #update_values
        }

        impl charybdis::ValueList for #struct_name {
            #serialized
        }

        impl charybdis::FromRow for #struct_name {
            #from_row
        }

        #find_model_query_rule
        #find_model_rule
        #find_first_model_rule
        #update_model_query_rule
        #partial_model_generator
    };

    TokenStream::from(expanded)
}

/// Generates the implementation of the MaterializedView trait
/// for the given struct.
#[proc_macro_attribute]
pub fn charybdis_view_model(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: CharybdisMacroArgs = parse_macro_input!(args);
    let mut input: DeriveInput = parse_macro_input!(input);
    let db_fields = CharybdisFields::from_input(&input).db_fields();

    strip_charybdis_attributes(&mut input);

    let struct_name = &input.ident;

    // Charybdis::MaterializedView consts
    let db_model_name_const = db_model_name_const(&args);
    let partition_keys_const = partition_keys_const(&args);
    let clustering_keys_const = clustering_keys_const(&args);
    let primary_key_const = primary_key_const(&args);
    let select_fields_clause = select_fields_clause(&args, &db_fields);
    let find_by_primary_key_query_const = find_by_primary_key_query_const(&args, &db_fields);
    let find_by_partition_key_query_const = find_by_partition_key_query_const(&args, &db_fields);

    // Charybdis::MaterializedView methods
    let primary_key_values = primary_key_values(&args);
    let partition_key_values = partition_key_values(&args);
    let clustering_key_values = clustering_key_values(&args);

    // Current model rules
    let find_model_query_rule = find_model_query_rule(&args, &db_fields, struct_name);

    // Associated functions for finding by  primary key
    let find_by_key_funs = find_by_primary_keys_functions(&args, &db_fields, struct_name);

    let expanded = quote! {
        #[derive(charybdis::ValueList, charybdis::FromRow)]
        #input

        impl #struct_name {
            #find_by_key_funs
        }

        impl charybdis::model::BaseModel for #struct_name {
            // consts
            #db_model_name_const
            #clustering_keys_const
            #partition_keys_const
            #primary_key_const
            #find_by_primary_key_query_const
            #find_by_partition_key_query_const
            #select_fields_clause
            // methods
            #primary_key_values
            #partition_key_values
            #clustering_key_values
        }

        impl charybdis::model::MaterializedView for #struct_name {}

        #find_model_query_rule
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn charybdis_udt_model(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let db_fields = CharybdisFields::from_input(&mut input).db_fields();

    let struct_name = &input.ident;

    // sort fields by name
    // https://github.com/scylladb/scylla-rust-driver/issues/370
    let mut sorted_fields: Vec<_> = db_fields.into_iter().collect();
    sorted_fields.sort_by(|a, b| a.ident.as_ref().unwrap().cmp(b.ident.as_ref().unwrap()));

    let gen = quote! {
        #[derive(charybdis::FromUserType, charybdis::IntoUserType, Clone)]
        pub struct #struct_name {
            #(#sorted_fields),*
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn char_model_field_attrs_gen(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: CharybdisMacroArgs = parse_macro_input!(args);
    let input: DeriveInput = parse_macro_input!(input);

    let tkn_2 = char_model_field_attrs_macro_gen(args, input);

    tkn_2.into()
}
