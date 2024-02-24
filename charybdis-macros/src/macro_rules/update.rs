use crate::utils::{camel_to_snake_case, where_placeholders};
use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse_str;

pub(crate) fn update_model_query_rule(
    struct_name: &Ident,
    args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let struct_name_str = camel_to_snake_case(&struct_name.to_string());
    let macro_name_str = format!("update_{}_query", struct_name_str);
    let macro_name = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let update = format!("UPDATE {} SET ", args.table_name());
    let query_str = format!(" WHERE {}", where_placeholders(&fields.primary_key_fields));

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
