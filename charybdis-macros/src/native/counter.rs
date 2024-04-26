use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_str;

use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::traits::CharybdisMacroArgs;

use crate::traits::fields::FieldsQuery;
use crate::traits::tuple::FieldsAsTuple;

pub(crate) fn increment_counter_methods(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> TokenStream {
    let increment_counter_methods: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            if !field.is_counter() {
                return None;
            }

            let increment_query = format!(
                "UPDATE {} SET {} = {} + ? WHERE {}",
                ch_args.table_name(),
                field.name,
                field.name,
                fields.primary_key_fields.where_placeholders()
            );

            let fun_name_str = format!("increment_{}", field.name);
            let fun_name = parse_str::<TokenStream>(&fun_name_str).unwrap();
            let types = fields.primary_key_fields.types();
            let values = fields.primary_key_fields.values();

            let expanded = quote! {
                pub fn #fun_name(
                    &self,
                    increment: i64,
                ) -> charybdis::query::CharybdisQuery<(charybdis::types::Counter, #(#types),*), Self, charybdis::query::ModelMutation> {
                    charybdis::query::CharybdisQuery::new(
                        #increment_query,
                        charybdis::query::QueryValue::Owned((charybdis::types::Counter(increment), #(#values),*)),
                    )
                }
            };

            Some(expanded)
        })
        .collect();

    let expanded = quote! {
        #(#increment_counter_methods)*
    };

    expanded
}

pub(crate) fn decrement_counter_methods(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> TokenStream {
    let decrement_counter_methods: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            if !field.is_counter() {
                return None;
            }

            let decrement_query = format!(
                "UPDATE {} SET {} = {} - ? WHERE {}",
                ch_args.table_name(),
                field.name,
                field.name,
                fields.primary_key_fields.where_placeholders()
            );

            let fun_name_str = format!("decrement_{}", field.name);
            let fun_name = parse_str::<TokenStream>(&fun_name_str).unwrap();
            let types = fields.primary_key_fields.types();
            let values = fields.primary_key_fields.values();

            let expanded = quote! {
                pub fn #fun_name(
                    &self,
                    decrement: i64,
                ) -> charybdis::query::CharybdisQuery<(charybdis::types::Counter, #(#types),*), Self, charybdis::query::ModelMutation> {
                    charybdis::query::CharybdisQuery::new(
                        #decrement_query,
                        charybdis::query::QueryValue::Owned((charybdis::types::Counter(decrement), #(#values),*)),
                    )
                }
            };

            Some(expanded)
        })
        .collect();

    let expanded = quote! {
        #(#decrement_counter_methods)*
    };

    expanded
}
