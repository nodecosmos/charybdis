use crate::utils::serialized_field_value_adder;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, ImplItem};

/// (check update_query_const.rs)
///
/// First we get all the non primary key fields used in set_fields clause then we get all primary key fields
/// used in where clause.
pub(crate) fn update_values(ch_args: &CharybdisMacroArgs, fields: &Vec<Field>) -> ImplItem {
    let mut primary_key = ch_args.primary_key();

    let mut update_values: Vec<String> = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .filter(|field| !primary_key.contains(field))
        .collect();

    update_values.append(primary_key.as_mut());

    let capacity: usize = update_values.len();
    let serialized_field_value_adder: TokenStream = serialized_field_value_adder(update_values);

    let generated = quote! {
        fn update_values(&self) -> charybdis::SerializedResult {
            let mut serialized = charybdis::SerializedValues::with_capacity(#capacity);

            #serialized_field_value_adder

            ::std::result::Result::Ok(::std::borrow::Cow::Owned(serialized))
        }
    };

    syn::parse_quote!(#generated)
}
