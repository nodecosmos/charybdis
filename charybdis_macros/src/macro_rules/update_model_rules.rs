use crate::utils::camel_to_snake_case;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse_str;

pub fn update_model_query_rule(args: &CharybdisMacroArgs, struct_name: &Ident) -> TokenStream {
    let table_name = args.table_name.as_ref().unwrap();
    let primary_key = args.primary_key();

    let struct_name_str = camel_to_snake_case(&struct_name.to_string());
    let macro_name_str = format!("update_{}_query", struct_name_str);
    let macro_name = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let primary_key_where_clause: String = primary_key.join(" = ? AND ");
    let update = format!("UPDATE {} SET ", table_name);
    let query_str = format!(" WHERE {} = ?", primary_key_where_clause);

    let expanded = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($query: literal) => {
                concat!(#update, $query, #query_str)
            }
        }

        pub(crate) use #macro_name;
    };

    expanded
}
