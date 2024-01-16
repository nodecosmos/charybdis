use charybdis_parser::fields::CharybdisFields;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_str;

// Here we utilize PUSH_{}_QUERY and PULL_{}_QUERY consts to generate Model functions
// for updating collection fields.
pub fn push_to_collection_funs(fields: &CharybdisFields) -> TokenStream {
    let push_to_collection_rules: Vec<TokenStream> = fields
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

            let field_name_upper = field_name.to_uppercase();

            let push_to_query_str = format!("Self::PUSH_{}_QUERY", field_name_upper);
            let push_to_query = parse_str::<TokenStream>(&push_to_query_str).unwrap();

            let fun_name_str = format!("push_{}", field_name);
            let fun_name = parse_str::<TokenStream>(&fun_name_str).unwrap();

            let expanded = quote! {
                pub async fn #fun_name(
                    &self,
                    session: &charybdis::CachingSession,
                    value: &impl charybdis::SerializeRow
                ) -> Result<charybdis::QueryResult, charybdis::errors::CharybdisError> {
                    let res = charybdis::operations::execute(session, #push_to_query, value).await?;

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

pub fn pull_from_collection_funs(fields: &CharybdisFields) -> TokenStream {
    let pull_from_collection_rules: Vec<TokenStream> = fields
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

            let field_name_upper = field_name.to_uppercase();

            let pull_from_query_str = format!("Self::PULL_{}_QUERY", field_name_upper);
            let pull_from_query = parse_str::<TokenStream>(&pull_from_query_str).unwrap();

            let fun_name_str = format!("pull_{}", field_name);
            let fun_name = parse_str::<TokenStream>(&fun_name_str).unwrap();

            let expanded = quote! {
                pub async fn #fun_name(
                    &self,
                    session: &charybdis::CachingSession,
                    value: &impl charybdis::SerializeRow
                ) -> Result<charybdis::QueryResult, charybdis::errors::CharybdisError> {

                    let res = charybdis::operations::execute(session, #pull_from_query, value).await?;

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
