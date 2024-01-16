use crate::utils::{args_to_pass, comma_sep_cols, struct_fields_to_fn_args, where_placeholders};
use charybdis_parser::fields::Field;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;

const MAX_FIND_BY_FUNCTIONS: usize = 3;

/// for up to 3 primary keys, generate find_by_primary_key functions
pub(crate) fn find_by_primary_keys_functions(
    ch_args: &CharybdisMacroArgs,
    fields: &Vec<Field>,
    struct_name: &syn::Ident,
) -> TokenStream {
    let table_name = ch_args.table_name();
    let comma_sep_cols = comma_sep_cols(fields);

    let primary_key_stack = ch_args.primary_key();
    let mut generated = quote! {};

    for i in 0..primary_key_stack.len() {
        if i == MAX_FIND_BY_FUNCTIONS {
            break;
        }

        let current_keys = primary_key_stack
            .iter()
            .take(i + 1)
            .map(|key| key.to_string())
            .collect::<Vec<String>>();

        let query_str = format!(
            "SELECT {} FROM {} WHERE {}",
            comma_sep_cols,
            table_name,
            where_placeholders(&current_keys)
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
        let is_complete_pk = current_keys.len() == primary_key_stack.len();
        let arguments = struct_fields_to_fn_args(struct_name.to_string(), fields.clone(), current_keys.clone());
        let args_to_pass = args_to_pass(current_keys.clone());
        let generated_func;

        if is_complete_pk {
            // for complete pk we get single row
            generated_func =
                find_one_generated_fn(&find_by_fun_name, &arguments, &args_to_pass, struct_name, query_str);
        } else {
            // for partial pk we get a stream
            generated_func =
                find_many_generated_fn(&find_by_fun_name, &arguments, &args_to_pass, struct_name, query_str);
        }

        generated.extend(generated_func);
    }

    generated
}

fn find_one_generated_fn(
    find_by_fun_name: &syn::Ident,
    arguments: &Vec<syn::FnArg>,
    args_to_pass: &Vec<syn::Ident>,
    struct_name: &syn::Ident,
    query_str: String,
) -> TokenStream {
    quote! {
        pub async fn #find_by_fun_name(
            session: &charybdis::CachingSession,
            #(#arguments),*
        ) -> Result<#struct_name, charybdis::errors::CharybdisError> {
            let query_result = session.execute(#query_str, (#(#args_to_pass),*,)).await?;
            let res = query_result.first_row_typed()?;

            Ok(res)
        }
    }
}

fn find_many_generated_fn(
    find_by_fun_name: &syn::Ident,
    arguments: &Vec<syn::FnArg>,
    args_to_pass: &Vec<syn::Ident>,
    struct_name: &syn::Ident,
    query_str: String,
) -> TokenStream {
    quote! {
        pub async fn #find_by_fun_name(
            session: &charybdis::CachingSession,
            #(#arguments),*
        ) -> Result<charybdis::stream::CharybdisModelStream<#struct_name>, charybdis::errors::CharybdisError> {
            let query_result = session.execute_iter(#query_str, (#(#args_to_pass),*,)).await?;
            let rows = query_result.into_typed::<Self>();

            Ok(charybdis::stream::CharybdisModelStream::from(rows))
        }
    }
}
