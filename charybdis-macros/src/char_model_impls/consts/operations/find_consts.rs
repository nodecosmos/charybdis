use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::{Field, ImplItem};

use crate::utils::comma_sep_cols;

pub(crate) fn find_by_primary_key_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let primary_key = ch_args.primary_key();
    let table_name = ch_args.table_name.clone().unwrap();

    let comma_sep_cols = comma_sep_cols(fields);
    let primary_key_where_clause: String = primary_key.join(" = ? AND ");

    let query_str = format!(
        "SELECT {} FROM {} WHERE {} = ?",
        comma_sep_cols, table_name, primary_key_where_clause
    );

    let generated = quote! {
        const FIND_BY_PRIMARY_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn find_by_partition_key_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let partition_keys = ch_args.partition_keys.clone().unwrap();
    let table_name = ch_args.table_name.clone().unwrap();
    let comma_sep_cols = comma_sep_cols(fields);

    let partition_keys_where_clause: String = partition_keys.join(" = ? AND ");

    let query_str = format!(
        "SELECT {} FROM {} WHERE {} = ?",
        comma_sep_cols, table_name, partition_keys_where_clause
    );

    let generated = quote! {
        const FIND_BY_PARTITION_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
