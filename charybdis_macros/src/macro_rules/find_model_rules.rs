use crate::utils::{camel_to_snake_case, comma_sep_cols};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_str, Field};

pub fn find_model_query_rule(args: &CharybdisMacroArgs, fields: &Vec<Field>, struct_name: &Ident) -> TokenStream {
    let comma_sep_cols = comma_sep_cols(fields);
    let table_name = args.table_name.clone().unwrap();
    let struct_name_str = camel_to_snake_case(&struct_name.to_string());

    let macro_name_str: String = format!("find_{}_query", struct_name_str);
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!("SELECT {} FROM {} WHERE ", comma_sep_cols, table_name);

    let expanded = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($query: literal) => {
                concat!(#query_str, $query)
            }
        }

        pub(crate) use #macro_name;
    };

    expanded
}

pub fn find_model_rule(args: &CharybdisMacroArgs, fields: &Vec<Field>, struct_name: &Ident) -> TokenStream {
    let comma_sep_cols = comma_sep_cols(fields);
    let table_name = args.table_name.clone().unwrap();
    let struct_name_str = camel_to_snake_case(&struct_name.to_string());

    let macro_name_str: String = format!("find_{}", struct_name_str);
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!("SELECT {} FROM {} WHERE ", comma_sep_cols, table_name);

    let expanded = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($session: ident, $query: literal, $values: expr) => {
               <#struct_name as charybdis::operations::Find>::find($session, concat!(#query_str, $query), $values)
            }
        }

        pub(crate) use #macro_name;
    };

    expanded
}

pub fn find_first_model_rule(args: &CharybdisMacroArgs, fields: &Vec<Field>, struct_name: &Ident) -> TokenStream {
    let comma_sep_cols = comma_sep_cols(fields);
    let table_name = args.table_name.clone().unwrap();
    let struct_name_str = camel_to_snake_case(&struct_name.to_string());

    let macro_name_str: String = format!("find_first_{}", struct_name_str);
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!("SELECT {} FROM {} WHERE ", comma_sep_cols, table_name);

    let expanded = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($session: ident, $query: literal, $values: expr) => {
                <#struct_name as charybdis::operations::Find>::find_first($session, concat!(#query_str, $query), $values)
            }
        }

        pub(crate) use #macro_name;
    };

    expanded
}
