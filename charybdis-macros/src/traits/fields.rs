mod arguments;
mod find;
mod hash_map;
mod query;

pub(crate) use arguments::*;
pub(crate) use find::*;
pub(crate) use hash_map::*;
pub(crate) use query::*;

use charybdis_parser::fields::Field;

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
