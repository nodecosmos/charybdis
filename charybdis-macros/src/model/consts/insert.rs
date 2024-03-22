use crate::traits::fields::FieldsQuery;

use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn insert_query_const(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> ImplItem {
    let query_str: String = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        ch_args.table_name(),
        fields.db_fields.comma_sep_cols(),
        fields.db_fields.insert_bind_markers(),
    );

    let generated = quote! {
        const INSERT_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn insert_if_not_exists_query_const(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> ImplItem {
    let query_str: String = format!(
        "INSERT INTO {} ({}) VALUES ({}) IF NOT EXISTS",
        ch_args.table_name(),
        fields.db_fields.comma_sep_cols(),
        fields.db_fields.insert_bind_markers(),
    );

    let generated = quote! {
        const INSERT_IF_NOT_EXIST_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
