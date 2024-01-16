use crate::utils::{set_bind_markers, where_bind_markers};
use charybdis_parser::fields::{CharFieldsExt, Field};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn update_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let query_str: String = format!(
        "UPDATE {} SET {} WHERE {}",
        ch_args.table_name(),
        set_bind_markers(fields.non_primary_key_fields()),
        where_bind_markers(fields.primary_key_fields()),
    );

    let generated = quote! {
        const UPDATE_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
