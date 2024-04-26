extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::traits::CharybdisMacroArgs;

use crate::model::*;
use crate::native::{
    decrement_counter_methods, delete_by_primary_key_functions, find_by_global_secondary_index,
    find_by_local_secondary_index, find_by_primary_keys_functions, increment_counter_methods,
    pull_from_collection_consts, pull_from_collection_methods, push_to_collection_consts, push_to_collection_methods,
};
use crate::rules::*;
use crate::scylla::from_row;

mod model;
mod native;
mod rules;

mod scylla;
mod traits;

#[proc_macro_attribute]
pub fn charybdis_model(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: CharybdisMacroArgs = parse_macro_input!(args);

    let mut input: DeriveInput = parse_macro_input!(input);
    let mut new_fields = CharybdisFields::from_input(&input, &args);
    let fields = new_fields.populate(&args);

    let struct_name = &input.ident.clone();

    // partial_<model_name>!(StructName, field1, field2, ...);
    let partial_model_generator = partial_model_macro_generator(&input, &args, &fields);

    // Charybdis::BaseModel types
    let primary_key_type = primary_key_type(&fields);
    let partition_key_type = partition_key_type(&fields);

    // Charybdis::BaseModel consts
    let db_model_name_const = db_model_name_const(&args);
    let find_by_primary_key_query_const = find_by_primary_key_query_const(&args, &fields);
    let find_by_partition_key_query_consts = find_by_partition_key_query_consts(&args, &fields);
    let find_first_by_partition_key_query_const = find_first_by_partition_key_query_const(&args, &fields);
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
    let push_to_collection_methods = push_to_collection_methods(&fields);
    let pull_from_collection_methods = pull_from_collection_methods(&fields);

    // Counter methods
    let increment_counter_methods = increment_counter_methods(&args, &fields);
    let decrement_counter_methods = decrement_counter_methods(&args, &fields);

    // FromRow trait
    let from_row = from_row(struct_name, &fields);

    // Current model macro rules
    let find_model_query_rule = find_model_query_rule(struct_name, &args, &fields);
    let find_model_rule = find_model_rule(struct_name, &args, &fields);
    let find_first_model_rule = find_first_model_rule(struct_name, &args, &fields);
    let update_model_query_rule = update_model_query_rule(struct_name, &args, &fields);
    let delete_model_query_rule = delete_model_query_rule(struct_name, &args);
    let delete_model_rule = delete_model_rule(struct_name, &args);

    // Associated functions
    let find_by_key_funs = find_by_primary_keys_functions(struct_name, &args, &fields);
    let find_by_local_secondary_index_funs = find_by_local_secondary_index(struct_name, &args, &fields);
    let find_by_global_secondary_index_funs = find_by_global_secondary_index(struct_name, &args, &fields);
    let delete_by_cks_funs = delete_by_primary_key_functions(&args, &fields);

    CharybdisFields::proxy_charybdis_attrs_to_scylla(&mut input);
    CharybdisFields::strip_charybdis_attributes(&mut input);

    let expanded = quote! {
        #[derive(charybdis::SerializeRow)]
        #input

        impl #struct_name {
            #find_by_key_funs
            #delete_by_cks_funs

            #find_by_local_secondary_index_funs
            #find_by_global_secondary_index_funs

            #push_to_collection_consts
            #pull_from_collection_consts

            // methods
            #push_to_collection_methods
            #pull_from_collection_methods
            #increment_counter_methods
            #decrement_counter_methods
        }

       impl charybdis::model::BaseModel for #struct_name {
            // types
            #primary_key_type
            #partition_key_type

            // consts
            #db_model_name_const
            #find_by_primary_key_query_const
            #find_by_partition_key_query_consts
            #find_first_by_partition_key_query_const

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
        #delete_model_query_rule
        #delete_model_rule
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn charybdis_view_model(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: CharybdisMacroArgs = parse_macro_input!(args);
    let mut input: DeriveInput = parse_macro_input!(input);
    let mut new_fields = CharybdisFields::from_input(&input, &args);
    let fields = new_fields.populate(&args);

    let struct_name = &input.ident.clone();

    // Charybdis::BaseModel types
    let primary_key_type = primary_key_type(&fields);
    let partition_key_type = partition_key_type(&fields);

    // Charybdis::MaterializedView consts
    let db_model_name_const = db_model_name_const(&args);
    let find_by_primary_key_query_const = find_by_primary_key_query_const(&args, &fields);
    let find_by_partition_key_query_consts = find_by_partition_key_query_consts(&args, &fields);
    let find_first_by_partition_key_query_const = find_first_by_partition_key_query_const(&args, &fields);

    // Charybdis::BaseModel methods
    let primary_key_values_method = primary_key_values_method(&fields);
    let partition_key_values_method = partition_key_values_method(&fields);

    // Current model rules
    let find_model_query_rule = find_model_query_rule(struct_name, &args, &fields);

    // Associated functions
    let find_by_key_funs = find_by_primary_keys_functions(struct_name, &args, &fields);

    CharybdisFields::proxy_charybdis_attrs_to_scylla(&mut input);
    CharybdisFields::strip_charybdis_attributes(&mut input);

    let expanded = quote! {
        #[derive(charybdis::SerializeRow)]
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
            #find_by_partition_key_query_consts
            #find_first_by_partition_key_query_const

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

    let gen = quote! {
        #[derive(charybdis::FromUserType, charybdis::SerializeCql)]
        #input
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
