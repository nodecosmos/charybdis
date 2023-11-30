use crate::utils::serialized_field_value_adder;
use quote::quote;
use syn::{Field, ImplItem};

pub(crate) fn serialized(db_fields: &Vec<Field>, all_fields: &Vec<Field>) -> ImplItem {
    let capacity: usize = db_fields.len() + all_fields.len();

    let str_vec = db_fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect::<Vec<String>>();

    let fields_adder = serialized_field_value_adder(str_vec);

    let generated = quote! {
        fn serialized(&self) -> charybdis::SerializedResult {
            let mut serialized = charybdis::SerializedValues::with_capacity(#capacity);

            #fields_adder

            ::std::result::Result::Ok(::std::borrow::Cow::Owned(serialized))
        }
    };

    syn::parse_quote!(#generated)
}
