use crate::utils::comma_sep_cols;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::{Field, ImplItem};

pub fn select_fields_clause(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let table_name = ch_args.table_name.clone().unwrap();

    let comma_sep_cols = comma_sep_cols(fields);

    let query_str = format!("SELECT {} FROM {}", comma_sep_cols, table_name);

    let generated = quote! {
        const SELECT_FIELDS_CLAUSE: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
