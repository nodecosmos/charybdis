use quote::quote;
use syn::ImplItem;

use charybdis_parser::fields::CharybdisFields;

use crate::traits::tuple::Tuple;

/// returns tuple of primary key types
pub(crate) fn primary_key_type(fields: &CharybdisFields) -> ImplItem {
    let types_tuple = fields.primary_key_fields.types_tp();

    let primary_key_type = quote! {
        type PrimaryKey = #types_tuple;
    };

    syn::parse_quote!(#primary_key_type)
}

pub(crate) fn partition_key_type(fields: &CharybdisFields) -> ImplItem {
    let types_tuple = fields.partition_key_fields.types_tp();

    let partition_key_type = quote! {
        type PartitionKey = #types_tuple;
    };

    syn::parse_quote!(#partition_key_type)
}

/// returns tuple of self values
/// e.g. (self.id, self.name)
pub(crate) fn primary_key_values_method(fields: &CharybdisFields) -> ImplItem {
    let values_tuple = fields.primary_key_fields.values_tp();

    let primary_key_values_method = quote! {
        fn primary_key_values(&self) -> Self::PrimaryKey {
            return #values_tuple;
        }
    };

    syn::parse_quote!(#primary_key_values_method)
}

pub(crate) fn partition_key_values_method(fields: &CharybdisFields) -> ImplItem {
    let values_tuple = fields.partition_key_fields.values_tp();

    let partition_key_values_method = quote! {
        fn partition_key_values(&self) -> Self::PartitionKey {
            return #values_tuple;
        }
    };

    syn::parse_quote!(#partition_key_values_method)
}
