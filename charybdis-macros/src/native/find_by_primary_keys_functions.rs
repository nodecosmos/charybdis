use crate::utils::{comma_sep_cols, serialized_value_adder, struct_fields_to_fn_args};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;

const MAX_FIND_BY_FUNCTIONS: usize = 3;

/// for up to 3 primary keys, generate find_by_primary_key functions
pub(crate) fn find_by_primary_keys_functions(
    ch_args: &CharybdisMacroArgs,
    fields: &Vec<Field>,
    struct_name: &syn::Ident,
) -> TokenStream {
    let table_name = ch_args.table_name.clone().unwrap();
    let comma_sep_cols = comma_sep_cols(fields);

    let mut primary_key_stack = ch_args.primary_key();
    let primary_key_len = primary_key_stack.len();
    let mut generated = quote! {};

    let mut i = 0;

    while !primary_key_stack.is_empty() {
        if i > MAX_FIND_BY_FUNCTIONS {
            break;
        }

        i += 1;

        let is_complete_pk = primary_key_stack.len() == primary_key_len;
        let current_keys = primary_key_stack.clone();
        let primary_key_where_clause: String = current_keys.join(" = ? AND ");
        let query_str = format!(
            "SELECT {} FROM {} WHERE {} = ?",
            comma_sep_cols, table_name, primary_key_where_clause
        );
        let find_by_fun_name_str = format!(
            "find_by_{}",
            current_keys
                .iter()
                .map(|key| key.to_string())
                .collect::<Vec<String>>()
                .join("_and_")
        );
        let find_by_fun_name = syn::Ident::new(&find_by_fun_name_str, proc_macro2::Span::call_site());

        let arguments = struct_fields_to_fn_args(struct_name.to_string(), fields.clone(), current_keys.clone());
        let capacity = current_keys.len();
        let serialized_adder = serialized_value_adder(current_keys.clone());
        let generated_func;

        if is_complete_pk {
            // for complete pk we get single row
            generated_func = find_one_generated_fn(
                &find_by_fun_name,
                &arguments,
                struct_name,
                capacity,
                serialized_adder,
                query_str,
            );
        } else {
            generated_func = find_many_generated_fn(
                &find_by_fun_name,
                &arguments,
                struct_name,
                capacity,
                serialized_adder,
                query_str,
            );
        }

        primary_key_stack.pop();

        generated.extend(generated_func);
    }

    generated
}

fn find_one_generated_fn(
    find_by_fun_name: &syn::Ident,
    arguments: &Vec<syn::FnArg>,
    struct_name: &syn::Ident,
    capacity: usize,
    serialized_adder: TokenStream,
    query_str: String,
) -> TokenStream {
    quote! {
        pub async fn #find_by_fun_name(
            session: &charybdis::CachingSession,
            #(#arguments),*
        ) -> Result<#struct_name, charybdis::errors::CharybdisError> {
            let mut serialized = charybdis::SerializedValues::with_capacity(#capacity);

            #serialized_adder

            let query_result = session.execute(#query_str, serialized).await?;
            let res = query_result.first_row_typed()?;

            Ok(res)
        }
    }
}

fn find_many_generated_fn(
    find_by_fun_name: &syn::Ident,
    arguments: &Vec<syn::FnArg>,
    struct_name: &syn::Ident,
    capacity: usize,
    serialized_adder: TokenStream,
    query_str: String,
) -> TokenStream {
    quote! {
        pub async fn #find_by_fun_name(
            session: &charybdis::CachingSession,
            #(#arguments),*
        ) -> Result<charybdis::stream::CharybdisModelStream<#struct_name>, charybdis::errors::CharybdisError> {
            use futures::TryStreamExt;

            let mut serialized = charybdis::SerializedValues::with_capacity(#capacity);

            #serialized_adder

            let query_result = session.execute_iter(#query_str, serialized).await?;
            let rows = query_result.into_typed::<Self>();

            Ok(charybdis::stream::CharybdisModelStream::from(rows))
        }
    }
}
