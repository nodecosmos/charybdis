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
    pub fn from_field(field: &syn::Field) -> Self {
        FieldAttributes::from_attributes(&field.attrs)
            .map(|char_attrs| {
                let ident = field.ident.clone().unwrap();
                return Field {
                    ident: ident.clone(),
                    ty: field.ty.clone(),
                    ty_path: match &field.ty {
                        syn::Type::Path(type_path) => type_path.clone(),
                        _ => panic!("Only type path is supported!"),
                    },
                    char_attrs,
                    attrs: field.attrs.clone(),
                    span: field.span(),
                    is_partition_key: true,
                    is_clustering_key: false,
                };
            })
            .unwrap()
    }

    pub fn is_primary_key(&self) -> bool {
        self.is_partition_key || self.is_clustering_key
    }
}

pub trait FieldsTypes {
    fn types(&self) -> Vec<syn::Type>;
}

impl FieldsTypes for &Vec<Field> {
    fn types(&self) -> Vec<syn::Type> {
        self.iter().map(|field| field.ty.clone()).collect()
    }
}

pub struct CharybdisFields {
    pub all_fields: Vec<Field>,
    pub partition_key_fields: Vec<Field>,
    pub clustering_key_fields: Vec<Field>,
    pub db_fields: Vec<Field>,
}

impl CharybdisFields {
    pub fn primary_key_fields(&self) -> Vec<Field> {
        self.partition_key_fields
            .iter()
            .chain(self.clustering_key_fields.iter())
            .cloned()
            .collect()
    }

    pub fn non_primary_key_fields(&self) -> Vec<Field> {
        self.all_fields
            .iter()
            .filter(|field| !field.is_primary_key())
            .cloned()
            .collect()
    }

    pub fn non_db_fields(&self) -> Vec<Field> {
        self.all_fields
            .iter()
            .filter(|field| field.char_attrs.ignore.unwrap_or(false))
            .cloned()
            .collect()
    }
}

impl CharybdisFields {
    pub fn new(named_fields: &FieldsNamed, args: &CharybdisMacroArgs) -> Self {
        let mut partition_key_fields = vec![];
        let mut clustering_key_fields = vec![];
        let mut db_fields = vec![];
        let mut all_fields = vec![];

        for key in args.partition_keys() {
            let field = named_fields
                .named
                .iter()
                .find(|f| f.ident.clone().unwrap().to_string() == key)
                .expect(&format!("Partition key {} not found in struct fields", key));

            let char_field = Field::from_field(field);

            partition_key_fields.push(char_field.clone());
            all_fields.push(char_field.clone());
            db_fields.push(char_field.clone());
        }

        for key in args.clustering_keys() {
            let field = named_fields
                .named
                .iter()
                .find(|f| f.ident.clone().unwrap().to_string() == key)
                .expect(&format!("Clustering key {} not found in struct fields", key));

            let char_field = Field::from_field(field);

            clustering_key_fields.push(char_field.clone());
            all_fields.push(char_field.clone());
            db_fields.push(char_field.clone());
        }

        for field in &named_fields.named {
            let char_field = Field::from_field(field);

            if !args.partition_keys().contains(&char_field.ident.to_string())
                && !args.clustering_keys().contains(&char_field.ident.to_string())
            {
                all_fields.push(char_field.clone());

                if !char_field.char_attrs.ignore.unwrap_or(false) {
                    db_fields.push(char_field.clone());
                }
            }
        }

        Self {
            partition_key_fields,
            clustering_key_fields,
            all_fields,
            db_fields,
        }
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
