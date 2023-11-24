mod hash_expr_lit_to_hash;
mod parse_fields_from_array;

use crate::macro_args::hash_expr_lit_to_hash::hash_expr_lit_to_hash;
use crate::macro_args::parse_fields_from_array::{parse_arr_expr_from_literals, parse_loc_sec_idx_array_expr};
use crate::schema::secondary_indexes::LocalIndexTarget;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};

#[derive(Debug, Default)]
pub struct CharybdisMacroArgs {
    pub table_name: Option<String>,
    pub type_name: Option<String>,
    pub base_table: Option<String>,
    pub partition_keys: Option<Vec<String>>,
    pub clustering_keys: Option<Vec<String>>,
    pub global_secondary_indexes: Option<Vec<String>>,
    pub local_secondary_indexes: Option<Vec<LocalIndexTarget>>,
    pub exclude_partial_model: Option<bool>,
    pub fields_names: Option<Vec<String>>,
    pub field_types_hash: Option<HashMap<String, TokenStream>>,
    pub field_attributes_hash: Option<HashMap<String, TokenStream>>,
    pub table_options: Option<String>,
}

impl CharybdisMacroArgs {
    pub fn primary_key(&self) -> Vec<String> {
        let mut primary_key: Vec<String> = self.partition_keys.clone().unwrap();
        let mut clustering_keys: Vec<String> = self.clustering_keys.clone().unwrap();

        primary_key.append(clustering_keys.as_mut());
        primary_key
    }
}

impl Parse for CharybdisMacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut type_name = None;
        let mut table_name = None;
        let mut base_table = None;
        let mut partition_keys = None;
        let mut clustering_keys = None;
        let mut global_secondary_indexes = None;
        let mut local_secondary_indexes = None;
        let mut fields_names = None;
        let mut field_types_hash = None;
        let mut field_attributes_hash = None;
        let mut table_options = None;
        let mut exclude_partial_model = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;

            match key.to_string().as_str() {
                "type_name" => {
                    let value: syn::Expr = input.parse()?;
                    type_name = Option::from(value.to_token_stream().to_string());
                }
                "table_name" => {
                    let value: syn::Expr = input.parse()?;
                    table_name = Option::from(value.to_token_stream().to_string());
                }
                "base_table" => {
                    let value: syn::Expr = input.parse()?;
                    base_table = Option::from(value.to_token_stream().to_string());
                }
                "partition_keys" => {
                    let array: syn::ExprArray = input.parse()?;
                    let parsed = parse_arr_expr_from_literals(array);

                    partition_keys = Some(parsed)
                }
                "clustering_keys" => {
                    let array: syn::ExprArray = input.parse()?;
                    let parsed = parse_arr_expr_from_literals(array);

                    clustering_keys = Some(parsed)
                }
                "global_secondary_indexes" => {
                    let array: syn::ExprArray = input.parse()?;
                    let parsed = parse_arr_expr_from_literals(array);

                    global_secondary_indexes = Some(parsed)
                }
                "local_secondary_indexes" => {
                    let array: syn::ExprArray = input.parse()?;
                    let parsed = parse_loc_sec_idx_array_expr(array);

                    local_secondary_indexes = Some(parsed)
                }
                "exclude_partial_model" => {
                    let value: syn::LitBool = input.parse()?;
                    exclude_partial_model = Option::from(value.value());
                }
                "fields_names" => {
                    let array: syn::ExprArray = input.parse()?;
                    let parsed = parse_arr_expr_from_literals(array);

                    fields_names = Some(parsed)
                }
                "field_types_hash" => {
                    let hash: syn::Expr = input.parse()?;
                    let parsed_field_types_hash = hash_expr_lit_to_hash(hash, "field_types_hash".to_string());

                    field_types_hash = Some(parsed_field_types_hash);
                }
                "field_attributes_hash" => {
                    // parse ruby style hash
                    let hash: syn::Expr = input.parse()?;
                    let parsed_field_attributes_hash = hash_expr_lit_to_hash(hash, "field_attributes_hash".to_string());
                    field_attributes_hash = Some(parsed_field_attributes_hash);
                }
                "table_options" => {
                    // parse ruby style hash
                    let value: syn::LitStr = input.parse()?;
                    table_options = Option::from(value.value());
                }
                _ => {}
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(CharybdisMacroArgs {
            type_name,
            table_name,
            base_table,
            partition_keys,
            clustering_keys,
            global_secondary_indexes,
            local_secondary_indexes,
            fields_names,
            field_types_hash,
            field_attributes_hash,
            table_options,
            exclude_partial_model,
        })
    }
}

impl From<TokenStream> for CharybdisMacroArgs {
    fn from(tokens: TokenStream) -> Self {
        // Convert the input tokens to a ParseStream
        let parse_stream: TokenStream = syn::parse2(tokens).unwrap();

        // Parse the ParseStream into a MyStruct instance
        let my_struct: CharybdisMacroArgs = syn::parse2(parse_stream).unwrap();

        // Return the parsed MyStruct instance
        my_struct
    }
}
