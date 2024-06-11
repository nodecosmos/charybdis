pub(crate) use arguments::*;
use charybdis_parser::fields::Field;
pub(crate) use find::*;
pub(crate) use hash_map::*;
use proc_macro2::TokenStream;
pub(crate) use query::*;
use quote::quote;

mod arguments;
mod find;
mod hash_map;

mod query;

pub(crate) trait ToIdents {
    fn to_idents(&self) -> Vec<syn::Ident>;
}

impl ToIdents for Vec<&Field<'_>> {
    fn to_idents(&self) -> Vec<syn::Ident> {
        self.iter()
            .map(|field| syn::Ident::new(&field.name, proc_macro2::Span::call_site()))
            .collect()
    }
}

pub(crate) trait FieldsNames {
    fn names(&self) -> Vec<String>;
}

impl FieldsNames for Vec<&Field<'_>> {
    fn names(&self) -> Vec<String> {
        self.iter().map(|field| field.name.clone()).collect()
    }
}

pub(crate) trait CollectionStatements {
    fn use_statement(&self) -> TokenStream;
    fn new_collection(&self) -> TokenStream;
    fn extend_statement(&self) -> TokenStream;
    fn remove_statement(&self) -> TokenStream;
}

impl CollectionStatements for Field<'_> {
    fn use_statement(&self) -> TokenStream {
        if self.is_list() {
            quote! { use std::vec::Vec; }
        } else if self.is_map() {
            quote! { use std::collections::HashMap; }
        } else if self.is_set() {
            quote! { use std::collections::HashSet; }
        } else {
            panic!("Not a collection type");
        }
    }

    fn new_collection(&self) -> TokenStream {
        if self.is_list() {
            quote! { Vec::new() }
        } else if self.is_map() {
            quote! { HashMap::new() }
        } else if self.is_set() {
            quote! { HashSet::new() }
        } else {
            panic!("Not a collection type");
        }
    }

    fn extend_statement(&self) -> TokenStream {
        let field_name = &self.ident;
        let new_collection = self.new_collection();

        return if self.is_option {
            quote! {
                self.#field_name.get_or_insert_with(#new_collection).extend(value);
            }
        } else {
            quote! {
                self.#field_name.extend(value);
            }
        };
    }

    fn remove_statement(&self) -> TokenStream {
        let field_name = &self.ident;
        let new_collection = self.new_collection();
        let field = quote! {
            self.#field_name.get_or_insert_with(#new_collection)
        };

        if self.is_list() || self.is_set() {
            quote! {
                #field.retain(|item| !value.contains(item));
            }
        } else if self.is_map() {
            quote! {
                value.iter().for_each(|(key, _)| {
                    #field.remove(key);
                });
            }
        } else {
            panic!("Not a collection type");
        }
    }
}
