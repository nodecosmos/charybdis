use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse_str;

use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::traits::string::ToSnakeCase;
use charybdis_parser::traits::CharybdisMacroArgs;

use crate::traits::fields::FieldsQuery;

pub(crate) fn find_model_query_rule(
    struct_name: &Ident,
    args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let macro_name_str: String = format!("find_{}_query", struct_name.to_string().to_snake_case());
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!(
        "SELECT {} FROM {} WHERE ",
        fields.db_fields.comma_sep_cols(),
        args.table_name()
    );

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

pub(crate) fn find_model_rule(struct_name: &Ident, args: &CharybdisMacroArgs, fields: &CharybdisFields) -> TokenStream {
    let macro_name_str: String = format!("find_{}", struct_name.to_string().to_snake_case());
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!(
        "SELECT {} FROM {} WHERE ",
        fields.db_fields.comma_sep_cols(),
        args.table_name()
    );

    let expanded = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($query: literal, $values: expr) => {
               <#struct_name as charybdis::operations::Find>::find(concat!(#query_str, $query), $values)
            }
        }

        pub(crate) use #macro_name;
    };

    expanded
}

pub(crate) fn find_first_model_rule(
    struct_name: &Ident,
    args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let macro_name_str: String = format!("find_first_{}", struct_name.to_string().to_snake_case());
    let macro_name: TokenStream = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let query_str = format!(
        "SELECT {} FROM {} WHERE ",
        fields.db_fields.comma_sep_cols(),
        args.table_name()
    );

    let expanded = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($query: literal, $values: expr) => {
                <#struct_name as charybdis::operations::Find>::find_first(concat!(#query_str, $query), $values)
            }
        }

        pub(crate) use #macro_name;
    };

    expanded
}
