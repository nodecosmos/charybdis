use charybdis_parser::fields::{CharybdisFields, Field, FieldsTypes};
use proc_macro2::TokenStream;
use quote::quote;
use syn::ImplItem;

/// returns tuple of primary key types
pub(crate) fn primary_key_type(fields: &CharybdisFields) -> ImplItem {
    let types_tuple = types_tuple(&fields.primary_key_fields);

    let primary_key_type = quote! {
        type PrimaryKey = #types_tuple;
    };

    syn::parse_quote!(#primary_key_type)
}

pub(crate) fn partition_key_type(fields: &CharybdisFields) -> ImplItem {
    let types_tuple = types_tuple(&fields.partition_key_fields);

    let partition_key_type = quote! {
        type PartitionKey = #types_tuple;
    };

    syn::parse_quote!(#partition_key_type)
}

/// returns tuple of self values
/// e.g. (self.id, self.name)
pub(crate) fn primary_key_values_method(fields: &CharybdisFields) -> ImplItem {
    let values_tuple = values_tuple(&fields.primary_key_fields);

    let primary_key_values_method = quote! {
        fn primary_key_values(&self) -> Self::PrimaryKey {
            return #values_tuple;
        }
    };

    syn::parse_quote!(#primary_key_values_method)
}

pub(crate) fn partition_key_values_method(fields: &CharybdisFields) -> ImplItem {
    let values_tuple = values_tuple(&fields.partition_key_fields);

    let partition_key_values_method = quote! {
        fn partition_key_values(&self) -> Self::PartitionKey {
            return #values_tuple;
        }
    };

    syn::parse_quote!(#partition_key_values_method)
}

fn types_tuple(fields: impl FieldsTypes) -> TokenStream {
    let types = fields.types();

    return if types.len() == 1 {
        let single_type = types.first().unwrap();
        quote! {
            (#single_type,)
        }
    } else {
        quote! {
            (#(#types),*)
        }
    };
}

fn values_tuple(fields: &Vec<Field>) -> TokenStream {
    let values = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.clone();
            quote! { self.#field_name.clone() }
        })
        .collect::<Vec<_>>();

    return if values.len() == 1 {
        let single_value = values.first().unwrap();
        quote! {
            (#single_value,)
        }
    } else {
        quote! {
            (#(#values),*)
        }
    };
}
