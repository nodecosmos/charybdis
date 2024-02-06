use crate::utils::where_placeholders;
use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_str;

pub(crate) fn push_to_collection_consts(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> TokenStream {
    let queries: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            if !field.is_list() && !field.is_set() {
                return None;
            }

            let query_str = format!(
                "UPDATE {} SET {} = {} + ? WHERE {}",
                ch_args.table_name(),
                field.name,
                field.name,
                where_placeholders(&fields.primary_key_fields),
            );

            let const_name = format!("PUSH_{}_QUERY", field.name.to_uppercase());
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

pub(crate) fn pull_from_collection_consts(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> TokenStream {
    let queries: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            if !field.is_list() && !field.is_set() {
                return None;
            }

            let query_str = format!(
                "UPDATE {} SET {} = {} - ? WHERE {}",
                ch_args.table_name(),
                field.name,
                field.name,
                where_placeholders(&fields.primary_key_fields),
            );

            let const_name = format!("PULL_{}_QUERY", field.name.to_uppercase());
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

pub(crate) fn push_to_collection_funs(fields: &CharybdisFields) -> TokenStream {
    let push_to_collection_rules: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            if !field.is_list() && !field.is_set() {
                return None;
            }

            let push_to_query_str = format!("Self::PUSH_{}_QUERY", field.name.to_uppercase());
            let push_to_query = parse_str::<TokenStream>(&push_to_query_str).unwrap();
            let fun_name_str = format!("push_{}", field.name);
            let fun_name = parse_str::<TokenStream>(&fun_name_str).unwrap();

            // Create tuple of primary key fields
            let pk_fields_tuple = fields.primary_key_fields.iter().map(|pk_field| {
                let pk_field_ident = &pk_field.ident;
                quote! { self.#pk_field_ident.clone() }
            });

            let expanded = quote! {
                pub async fn #fun_name(
                    &self,
                    session: &charybdis::CachingSession,
                    value: &impl charybdis::SerializeCql
                ) -> Result<charybdis::QueryResult, charybdis::errors::CharybdisError> {
                    let res = charybdis::operations::execute(
                        session,
                        #push_to_query,
                        (value, #(#pk_fields_tuple),*)
                    ).await?;

                    Ok(res)
                }
            };

            Some(expanded)
        })
        .collect();

    let expanded = quote! {
        #(#push_to_collection_rules)*
    };

    expanded
}

pub(crate) fn pull_from_collection_funs(fields: &CharybdisFields) -> TokenStream {
    let pull_from_collection_rules: Vec<TokenStream> = fields
        .db_fields
        .iter()
        .filter_map(|field| {
            if !field.is_list() && !field.is_set() {
                return None;
            }

            let pull_from_query_str = format!("Self::PULL_{}_QUERY", field.name.to_uppercase());
            let pull_from_query = parse_str::<TokenStream>(&pull_from_query_str).unwrap();

            let fun_name_str = format!("pull_{}", field.name);
            let fun_name = parse_str::<TokenStream>(&fun_name_str).unwrap();

            let pk_fields_tuple = fields.primary_key_fields.iter().map(|pk_field| {
                let pk_field_ident = &pk_field.ident;
                quote! { self.#pk_field_ident.clone() }
            });

            let expanded = quote! {
                pub async fn #fun_name(
                    &self,
                    session: &charybdis::CachingSession,
                    value: &impl charybdis::SerializeCql
                ) -> Result<charybdis::QueryResult, charybdis::errors::CharybdisError> {

                    let res = charybdis::operations::execute(
                        session,
                        #pull_from_query,
                        (value, #(#pk_fields_tuple),*))
                    .await?;

                    Ok(res)
                }
            };

            Some(expanded)
        })
        .collect();

    let expanded = quote! {
        #(#pull_from_collection_rules)*
    };

    expanded
}
