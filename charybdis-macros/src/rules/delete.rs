use crate::traits::string::ToSnakeCase;
use charybdis_parser::traits::CharybdisMacroArgs;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse_str;

pub(crate) fn delete_model_query_rule(struct_name: &Ident, args: &CharybdisMacroArgs) -> TokenStream {
    let macro_name_str: String = format!("delete_{}_query", struct_name.to_string().to_snake_case());
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!("DELETE FROM {} WHERE ", args.table_name());

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

pub(crate) fn delete_model_rule(struct_name: &Ident, args: &CharybdisMacroArgs) -> TokenStream {
    let macro_name_str: String = format!("delete_{}", struct_name.to_string().to_snake_case());
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!("DELETE FROM {} WHERE ", args.table_name());

    let expanded = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($query: literal, $values: expr) => {
               <#struct_name as charybdis::operations::Delete>::delete_by_query(concat!(#query_str, $query), $values)
            }
        }

        pub(crate) use #macro_name;
    };

    expanded
}
