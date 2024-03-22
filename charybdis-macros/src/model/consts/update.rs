use crate::traits::fields::FieldsQuery;
use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn update_query_const(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> ImplItem {
    let query_str: String = format!(
        "UPDATE {} SET {} WHERE {}",
        ch_args.table_name(),
        fields.non_primary_key_db_fields().set_bind_markers(),
        fields.primary_key_fields.where_bind_markers(),
    );

    let generated = quote! {
        const UPDATE_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
