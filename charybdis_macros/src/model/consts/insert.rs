use charybdis_parser::fields::Field;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

use crate::utils::{comma_sep_cols, insert_bind_markers};

pub(crate) fn insert_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let query_str: String = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        ch_args.table_name(),
        comma_sep_cols(fields),
        insert_bind_markers(fields),
    );

    let generated = quote! {
        const INSERT_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn insert_if_not_exists_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let query_str: String = format!(
        "INSERT INTO {} ({}) VALUES ({}) IF NOT EXISTS",
        ch_args.table_name(),
        comma_sep_cols(fields),
        insert_bind_markers(fields),
    );

    let generated = quote! {
        const INSERT_IF_NOT_EXIST_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
