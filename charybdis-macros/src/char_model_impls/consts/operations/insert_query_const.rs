use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::{Field, ImplItem};

use crate::utils::comma_sep_cols;

pub(crate) fn insert_query_const(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let table_name = ch_args.table_name.as_ref().unwrap();
    let comma_sep_cols = comma_sep_cols(fields);
    let coma_sep_values_placeholders: String = fields
        .iter()
        .map(|_| "?".to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let query_str: String = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name, comma_sep_cols, coma_sep_values_placeholders,
    );

    let generated = quote! {
        const INSERT_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
