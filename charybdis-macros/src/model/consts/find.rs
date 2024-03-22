use crate::traits::fields::FieldsQuery;

use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

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

pub(crate) fn find_by_partition_key_query_const(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> ImplItem {
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
