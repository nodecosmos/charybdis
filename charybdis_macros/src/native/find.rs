use crate::utils::{comma_sep_cols, struct_fields_to_fn_args, where_placeholders, Tuple};
use charybdis_parser::fields::{CharybdisFields, Field};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;

const MAX_FIND_BY_FUNCTIONS: usize = 3;

/// for up to 3 primary keys, generate find_by_primary_key functions
pub(crate) fn find_by_primary_keys_functions(
    struct_name: &syn::Ident,
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let table_name = ch_args.table_name();
    let comma_sep_cols = comma_sep_cols(&fields.db_fields);

    let primary_key_stack = &fields.primary_key_fields;
    let mut generated = quote! {};

    for i in 0..primary_key_stack.len() {
        if i == MAX_FIND_BY_FUNCTIONS {
            break;
        }

        let current_fields = primary_key_stack.iter().take(i + 1).cloned().collect::<Vec<Field>>();
        let current_field_names = current_fields
            .iter()
            .map(|field| field.name.clone())
            .collect::<Vec<String>>();

        let query_str = format!(
            "SELECT {} FROM {} WHERE {}",
            comma_sep_cols,
            table_name,
            where_placeholders(&current_fields)
        );
        let find_by_fun_name_str = format!("find_by_{}", current_field_names.join("_and_"));

        let find_by_fun_name = syn::Ident::new(&find_by_fun_name_str, proc_macro2::Span::call_site());
        let is_complete_pk = current_fields.len() == primary_key_stack.len();
        let arguments = struct_fields_to_fn_args(
            struct_name.to_string(),
            fields.db_fields.clone(),
            current_field_names.clone(),
        );
        let generated_func;

        if is_complete_pk {
            // for complete pk we get single row
            generated_func = find_one_generated_fn(&find_by_fun_name, &arguments, struct_name, query_str);
        } else {
            // for partial pk we get a stream
            generated_func = find_many_generated_fn(&find_by_fun_name, &arguments, struct_name, query_str);
        }

        generated.extend(generated_func);
    }

    generated
}

fn find_one_generated_fn(
    find_by_fun_name: &syn::Ident,
    arguments: &Vec<syn::FnArg>,
    struct_name: &syn::Ident,
    query_str: String,
) -> TokenStream {
    let types_tp = arguments.types_tp();
    let values_tp = arguments.values_tp();

    quote! {
        pub fn #find_by_fun_name<'a>(
            #(#arguments),*
        ) -> charybdis::query::CharybdisQuery<'a, #types_tp, Self, charybdis::query::ModelRow<Self>> {
            <#struct_name as charybdis::operations::Find>::find_first(#query_str, #values_tp)
        }
    }
}

fn find_many_generated_fn(
    find_by_fun_name: &syn::Ident,
    arguments: &Vec<syn::FnArg>,
    struct_name: &syn::Ident,
    query_str: String,
) -> TokenStream {
    let types_tp = arguments.types_tp();
    let values_tp = arguments.values_tp();

    quote! {
        pub fn #find_by_fun_name<'a>(
            #(#arguments),*
        ) -> charybdis::query::CharybdisQuery<'a, #types_tp, Self, charybdis::query::ModelStream<Self>> {
            <#struct_name as charybdis::operations::Find>::find(#query_str, #values_tp)
        }
    }
}
