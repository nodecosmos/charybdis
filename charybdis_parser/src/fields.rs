use crate::macro_args::CharybdisMacroArgs;
use darling::FromAttributes;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, FieldsNamed};

#[derive(FromAttributes, Clone)]
#[darling(attributes(charybdis))]
pub struct FieldAttributes {
    #[darling(default)]
    pub ignore: Option<bool>,
}

#[derive(Clone)]
pub struct Field {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub ty_path: syn::TypePath,
    pub char_attrs: FieldAttributes,
    pub attrs: Vec<syn::Attribute>,
    pub span: proc_macro2::Span,
    pub is_partition_key: bool,
    pub is_clustering_key: bool,
}

impl Field {
    pub fn is_primary_key(&self) -> bool {
        self.is_partition_key || self.is_clustering_key
    }
}

pub trait CharFieldsExt<T> {
    fn primary_key_fields(&self) -> Vec<&T>;
    fn partition_key_fields(&self) -> Vec<&T>;
    fn clustering_key_fields(&self) -> Vec<&T>;
    fn non_primary_key_fields(&self) -> Vec<&T>;
}

impl CharFieldsExt<Field> for Vec<Field> {
    fn primary_key_fields(&self) -> Vec<&Field> {
        self.iter().filter(|field| field.is_primary_key()).collect()
    }

    fn partition_key_fields(&self) -> Vec<&Field> {
        self.iter().filter(|field| field.is_partition_key).collect()
    }

    fn clustering_key_fields(&self) -> Vec<&Field> {
        self.iter().filter(|field| field.is_clustering_key).collect()
    }

    fn non_primary_key_fields(&self) -> Vec<&Field> {
        self.iter().filter(|field| !field.is_primary_key()).collect()
    }
}

pub trait TypesExt {
    fn types(&self) -> Vec<syn::Type>;
}

impl TypesExt for Vec<&Field> {
    fn types(&self) -> Vec<syn::Type> {
        self.iter().map(|field| field.ty.clone()).collect()
    }
}

pub struct CharybdisFields {
    pub all_fields: Vec<Field>,
    pub db_fields: Vec<Field>,
}

impl CharybdisFields {
    pub fn new(named_fields: &FieldsNamed, args: &CharybdisMacroArgs) -> Self {
        let all_fields: Vec<Field> = named_fields
            .named
            .iter()
            .map(|f| {
                FieldAttributes::from_attributes(&f.attrs).map(|char_attrs| {
                    let ident = f.ident.clone().unwrap();
                    return Field {
                        ident: ident.clone(),
                        ty: f.ty.clone(),
                        ty_path: match &f.ty {
                            syn::Type::Path(type_path) => type_path.clone(),
                            _ => panic!("Only type path is supported!"),
                        },
                        char_attrs,
                        attrs: f.attrs.clone(),
                        span: f.span(),
                        is_partition_key: args.partition_keys().contains(&ident.to_string()),
                        is_clustering_key: args.clustering_keys().contains(&ident.to_string()),
                    };
                })
            })
            .collect::<Result<_, _>>()
            .unwrap();

        let db_fields = all_fields
            .iter()
            .filter(|field| !field.char_attrs.ignore.unwrap_or(false))
            .cloned()
            .collect();

        Self { all_fields, db_fields }
    }

    pub fn from_input(input: &DeriveInput, args: &CharybdisMacroArgs) -> Self {
        match &input.data {
            Data::Struct(data) => match &data.fields {
                Fields::Named(named_fields) => Self::new(named_fields, args),
                _ => panic!("#[charybdis_model] works only for structs with named fields!"),
            },
            _ => panic!("#[charybdis_model] works only on structs!"),
        }
    }

    /// Map charybdis(ignore) to scylla(skip) and leave other attributes as is
    pub fn proxy_charybdis_attrs_to_scylla(input: &mut DeriveInput) {
        if let Data::Struct(data_struct) = &mut input.data {
            if let Fields::Named(fields_named) = &mut data_struct.fields {
                for field in &mut fields_named.named {
                    if let Some(ignore) = &FieldAttributes::from_attributes(&field.attrs).unwrap().ignore {
                        if *ignore {
                            field.attrs.push(syn::parse_quote!(#[scylla(skip)]));
                        }
                    }
                }
            }
        }
    }

    pub fn strip_charybdis_attributes(input: &mut DeriveInput) {
        if let Data::Struct(data_struct) = &mut input.data {
            if let Fields::Named(fields_named) = &mut data_struct.fields {
                for field in &mut fields_named.named {
                    field.attrs.retain(|attr| !attr.path().is_ident("charybdis"));
                }
            }
        }
    }
}
