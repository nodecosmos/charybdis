use crate::utils::where_placeholders;
use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_str;

pub(crate) fn push_to_collection_fields_query_consts(
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let queries: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.to_string();
            let field_type = field.ty.to_token_stream().to_string();

            let is_list = field_type.contains("List");
            let is_set = field_type.contains("Set");

            if !is_list && !is_set {
                return None;
            }

            let query_str = format!(
                "UPDATE {} SET {} = {} + ? WHERE {}",
                ch_args.table_name(),
                field_name,
                field_name,
                where_placeholders(&fields.primary_key_fields()),
            );

            let field_name_upper = field_name.to_uppercase();
            let const_name = format!("PUSH_{}_QUERY", field_name_upper);
            let const_name: TokenStream = parse_str::<TokenStream>(&const_name).unwrap();

            let expanded = quote! {
                pub const #const_name: &'static str = #query_str;
            };

            Some(expanded)
        })
        .collect();

    let expanded = quote! {
        #(#queries)*
    };

    expanded
}

pub(crate) fn pull_from_collection_fields_query_consts(
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let queries: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.to_string();
            let field_type = field.ty.to_token_stream().to_string();

            let is_list = field_type.contains("List");
            let is_set = field_type.contains("Set");

            if !is_list && !is_set {
                return None;
            }

            let query_str = format!(
                "UPDATE {} SET {} = {} - ? WHERE {}",
                ch_args.table_name(),
                field_name,
                field_name,
                where_placeholders(&fields.primary_key_fields()),
            );

            let field_name_upper = field_name.to_uppercase();
            let const_name = format!("PULL_{}_QUERY", field_name_upper);
            let const_name: TokenStream = parse_str::<TokenStream>(&const_name).unwrap();

            let expanded = quote! {
                pub const #const_name: &'static str = #query_str;
            };

            Some(expanded)
        })
        .collect();

    let expanded = quote! {
        #(#queries)*
    };

    expanded
}
