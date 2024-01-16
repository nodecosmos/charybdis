use crate::utils::where_placeholders;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn delete_query_const(ch_args: &CharybdisMacroArgs) -> ImplItem {
    let query_str: String = format!(
        "DELETE FROM {} WHERE {}",
        ch_args.table_name(),
        where_placeholders(&ch_args.primary_key()),
    );

    let generated = quote! {
        const DELETE_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn delete_by_partition_key_query_const(ch_args: &CharybdisMacroArgs) -> ImplItem {
    let query_str: String = format!(
        "DELETE FROM {} WHERE {}",
        ch_args.table_name(),
        where_placeholders(&ch_args.partition_keys()),
    );

    let generated = quote! {
        const DELETE_BY_PARTITION_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
