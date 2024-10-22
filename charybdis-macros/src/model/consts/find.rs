use quote::quote;
use syn::ImplItem;

use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::traits::CharybdisMacroArgs;

use crate::traits::fields::FieldsQuery;

pub(crate) fn find_all_query_const(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> ImplItem {
    let query_str = format!(
        "SELECT {} FROM {}",
        fields.db_fields.comma_sep_cols(),
        ch_args.table_name()
    );

    let generated = quote! {
        const FIND_ALL_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn find_by_primary_key_query_const(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> ImplItem {
    let query_str = format!(
        "SELECT {} FROM {} WHERE {}",
        fields.db_fields.comma_sep_cols(),
        ch_args.table_name(),
        fields.primary_key_fields.where_placeholders(),
    );

    let generated = quote! {
        const FIND_BY_PRIMARY_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn find_by_partition_key_query_consts(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> ImplItem {
    let query_str = format!(
        "SELECT {} FROM {} WHERE {}",
        fields.db_fields.comma_sep_cols(),
        ch_args.table_name(),
        fields.partition_key_fields.where_placeholders(),
    );

    let generated = quote! {
        const FIND_BY_PARTITION_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn find_first_by_partition_key_query_const(
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> ImplItem {
    let query_str = format!(
        "SELECT {} FROM {} WHERE {} LIMIT 1",
        fields.db_fields.comma_sep_cols(),
        ch_args.table_name(),
        fields.partition_key_fields.where_placeholders(),
    );

    let generated = quote! {
        const FIND_FIRST_BY_PARTITION_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
