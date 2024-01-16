use crate::utils::{comma_sep_cols, where_placeholders};
use charybdis_parser::fields::Field;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn find_by_primary_key_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let query_str = format!(
        "SELECT {} FROM {} WHERE {}",
        comma_sep_cols(fields),
        ch_args.table_name(),
        where_placeholders(&ch_args.primary_key()),
    );

    let generated = quote! {
        const FIND_BY_PRIMARY_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn find_by_partition_key_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let query_str = format!(
        "SELECT {} FROM {} WHERE {}",
        comma_sep_cols(fields),
        ch_args.table_name(),
        where_placeholders(&ch_args.partition_keys())
    );

    let generated = quote! {
        const FIND_BY_PARTITION_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
