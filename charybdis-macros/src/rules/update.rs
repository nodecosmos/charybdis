use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse_str;

use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::traits::string::ToSnakeCase;
use charybdis_parser::traits::CharybdisMacroArgs;

use crate::traits::fields::FieldsQuery;

pub(crate) fn update_model_query_rule(
    struct_name: &Ident,
    args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let struct_name_str = struct_name.to_string().to_snake_case();
    let macro_name_str = format!("update_{}_query", struct_name_str);
    let macro_name = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let update = format!("UPDATE {} SET ", args.table_name());
    let query_str = format!(" WHERE {}", fields.primary_key_fields.where_placeholders());

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
