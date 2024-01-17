extern crate proc_macro;

mod macro_rules;
mod model;
mod native;
mod scylla;
mod utils;

use crate::macro_rules::*;
use crate::model::*;
use crate::native::{
    delete_by_primary_key_functions, find_by_primary_keys_functions, pull_from_collection_consts,
    pull_from_collection_funs, push_to_collection_consts, push_to_collection_funs,
};
use crate::scylla::from_row;
use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::{parse_macro_input, Data, Fields};

/// This macro generates the implementation of the [Model] trait for the given struct.
#[proc_macro_attribute]
pub fn charybdis_model(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: CharybdisMacroArgs = parse_macro_input!(args);
    let mut input: DeriveInput = parse_macro_input!(input);
    let fields = CharybdisFields::from_input(&input, &args);

    CharybdisFields::proxy_charybdis_attrs_to_scylla(&mut input);
    CharybdisFields::strip_charybdis_attributes(&mut input);

    let struct_name = &input.ident;

    // Charybdis::BaseModel types
    let primary_key_type = primary_key_type(&fields);
    let partition_key_type = partition_key_type(&fields);

    // Charybdis::BaseModel consts
    let db_model_name_const = db_model_name_const(&args);
    let find_by_primary_key_query_const = find_by_primary_key_query_const(&args, &fields);
    let find_by_partition_key_query_const = find_by_partition_key_query_const(&args, &fields);
    let insert_query_const = insert_query_const(&args, &fields);

    // Charybdis::Model consts
    let insert_if_not_exists_query_const = insert_if_not_exists_query_const(&args, &fields);
    let update_query_const = update_query_const(&args, &fields);
    let delete_query_const = delete_query_const(&args, &fields);
    let delete_by_partition_key_query_const = delete_by_partition_key_query_const(&args, &fields);

    // Charybdis::BaseModel methods
    let primary_key_values_method = primary_key_values_method(&fields);
    let partition_key_values_method = partition_key_values_method(&fields);

    // Collection consts
    let push_to_collection_consts = push_to_collection_consts(&args, &fields);
    let pull_from_collection_consts = pull_from_collection_consts(&args, &fields);

    // Collection methods
    let push_to_collection_funs = push_to_collection_funs(&fields);
    let pull_from_collection_funs = pull_from_collection_funs(&fields);

    // FromRow trait
    let from_row = from_row(struct_name, &fields);

    // Current model rules
    let find_model_query_rule = find_model_query_rule(struct_name, &args, &fields);
    let find_model_rule = find_model_rule(struct_name, &args, &fields);
    let find_first_model_rule = find_first_model_rule(struct_name, &args, &fields);
    let update_model_query_rule = update_model_query_rule(struct_name, &args, &fields);

    // Associated functions for finding by primary key
    let find_by_key_funs = find_by_primary_keys_functions(struct_name, &args, &fields);
    let delete_by_cks_funs = delete_by_primary_key_functions(struct_name, &args, &fields);

    let partial_model_generator = partial_model_macro_generator(args, &input);

    let expanded = quote! {
        #[derive(charybdis::SerializeRow)]
        #input

        impl #struct_name {
            #find_by_key_funs
            #delete_by_cks_funs

            #push_to_collection_consts
            #pull_from_collection_consts

            // methods
            #push_to_collection_funs
            #pull_from_collection_funs
        }

       impl charybdis::model::BaseModel for #struct_name {
            // types
            #primary_key_type
            #partition_key_type

            // consts
            #db_model_name_const
            #find_by_primary_key_query_const
            #find_by_partition_key_query_const

            // methods
            #primary_key_values_method
            #partition_key_values_method
        }

        impl charybdis::model::Model for #struct_name {
            // operation consts
            #insert_query_const
            #insert_if_not_exists_query_const
            #update_query_const
            #delete_query_const
            #delete_by_partition_key_query_const
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
    let fields = CharybdisFields::from_input(&input, &args);

    CharybdisFields::strip_charybdis_attributes(&mut input);

    let struct_name = &input.ident;

    // Charybdis::BaseModel types
    let primary_key_type = primary_key_type(&fields);
    let partition_key_type = partition_key_type(&fields);

    // Charybdis::MaterializedView consts
    let db_model_name_const = db_model_name_const(&args);
    let find_by_primary_key_query_const = find_by_primary_key_query_const(&args, &fields);
    let find_by_partition_key_query_const = find_by_partition_key_query_const(&args, &fields);

    // Charybdis::BaseModel methods
    let primary_key_values_method = primary_key_values_method(&fields);
    let partition_key_values_method = partition_key_values_method(&fields);

    // Current model rules
    let find_model_query_rule = find_model_query_rule(struct_name, &args, &fields);

    // Associated functions for finding by  primary key
    let find_by_key_funs = find_by_primary_keys_functions(struct_name, &args, &fields);

    let expanded = quote! {
        #[derive(charybdis::FromRow)]
        #input

        impl #struct_name {
            #find_by_key_funs
        }

        impl charybdis::model::BaseModel for #struct_name {
            // types
            #primary_key_type
            #partition_key_type

            // consts
            #db_model_name_const
            #find_by_primary_key_query_const
            #find_by_partition_key_query_const

            // methods
            #primary_key_values_method
            #partition_key_values_method
        }

        impl charybdis::model::MaterializedView for #struct_name {}

        #find_model_query_rule
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn charybdis_udt_model(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named_fields) => named_fields,
            _ => panic!("#[charybdis_model] works only for structs with named fields!"),
        },
        _ => panic!("#[charybdis_model] works only on structs!"),
    };

    let struct_name = &input.ident;

    // sort fields by name
    // https://github.com/scylladb/scylla-rust-driver/issues/370
    let mut sorted_fields: Vec<_> = fields.named.iter().collect();
    sorted_fields.sort_by(|a, b| a.ident.cmp(&b.ident));

    let gen = quote! {
        #[derive(charybdis::FromUserType, Clone)]
        #[derive(charybdis::SerializeCql)]
        #[scylla(flavor = "enforce_order", skip_name_checks)]
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
