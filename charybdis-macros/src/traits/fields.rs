use crate::traits::r#type::TypeWithoutOptions;
use charybdis_parser::fields::Field;
use quote::quote;
use syn::{parse_str, Attribute};

pub(crate) trait QueryFields {
    fn comma_sep_cols(&self) -> String;
    fn insert_bind_markers(&self) -> String;
    fn set_bind_markers(&self) -> String;
    fn where_placeholders(&self) -> String;
    fn where_bind_markers(&self) -> String;
}

impl QueryFields for Vec<Field> {
    fn comma_sep_cols(&self) -> String {
        self.iter()
            .map(|field| field.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn insert_bind_markers(&self) -> String {
        let str_vec = self
            .iter()
            .map(|field| format!(":{}", field.name))
            .collect::<Vec<String>>()
            .join(", ");

        return str_vec;
    }

    fn set_bind_markers(&self) -> String {
        self.iter()
            .map(|field| format!("{} = :{}", field.name, field.name))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn where_placeholders(&self) -> String {
        self.iter()
            .map(|field| format!("{} = ?", field.name))
            .collect::<Vec<String>>()
            .join(" AND ")
    }

    fn where_bind_markers(&self) -> String {
        self.iter()
            .map(|field| format!("{} = :{}", field.name, field.name))
            .collect::<Vec<String>>()
            .join(" AND ")
    }
}

pub(crate) trait ToIdents {
    fn to_idents(&self) -> Vec<syn::Ident>;
}

impl ToIdents for Vec<Field> {
    fn to_idents(&self) -> Vec<syn::Ident> {
        self.iter()
            .map(|field| syn::Ident::new(&field.name, proc_macro2::Span::call_site()))
            .collect()
    }
}

pub(crate) trait Names {
    fn names(&self) -> Vec<String>;
}

impl Names for Vec<Field> {
    fn names(&self) -> Vec<String> {
        self.iter().map(|field| field.name.clone()).collect()
    }
}

pub(crate) trait FieldsToArguments {
    fn to_fn_args(&self) -> Vec<syn::FnArg>;
}

impl FieldsToArguments for Vec<Field> {
    fn to_fn_args(&self) -> Vec<syn::FnArg> {
        self.iter()
            .map(|field| {
                let type_wo_options = field.ty.type_without_options();
                parse_str::<syn::FnArg>(&format!("{}: {}", field.name, type_wo_options)).unwrap()
            })
            .collect::<Vec<syn::FnArg>>()
    }
}

/// It is used by `partial_model_macro_generator` to associate field **types**
/// and field **attributes** with field names for the generated partial model.
pub(crate) trait FieldHashMapString {
    /// key is field name and value is field type.
    fn field_types_hashmap_string(&self) -> String;

    /// key is field name and value is field attributes.
    fn field_attributes_hashmap_string(&self) -> String;
}

impl FieldHashMapString for Vec<Field> {
    fn field_types_hashmap_string(&self) -> String {
        let mut field_types = quote! {};

        for field in self.iter() {
            let field_ident = &field.ident;
            let ty = &field.ty;

            field_types.extend(quote! { #field_ident => #ty; });
        }

        field_types.to_string().replace('\n', "")
    }

    fn field_attributes_hashmap_string(&self) -> String {
        let mut field_attributes = quote! {};

        for field in self.iter() {
            let field_ident = &field.ident;
            let attrs: &Vec<Attribute> = &field.attrs;

            field_attributes.extend(quote! { #field_ident => #(#attrs)*; });
        }

        // strip newlines
        field_attributes.to_string().replace('\n', "")
    }
}
