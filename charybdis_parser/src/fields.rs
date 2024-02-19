use crate::macro_args::CharybdisMacroArgs;
use darling::FromAttributes;
use quote::ToTokens;
use std::fmt::{Display, Formatter};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, FieldsNamed};

pub enum Types {
    Ascii,
    BigInt,
    Blob,
    Boolean,
    Counter,
    Date,
    Decimal,
    Double,
    Duration,
    Float,
    Inet,
    Int,
    SmallInt,
    Text,
    Time,
    Timestamp,
    Timeuuid,
    TinyInt,
    Uuid,
    Varchar,
    Varint,
    Map,
    List,
    Set,
    Tuple,
    Frozen,
}

impl Display for Types {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Types::Ascii => write!(f, "Ascii"),
            Types::BigInt => write!(f, "BigInt"),
            Types::Blob => write!(f, "Blob"),
            Types::Boolean => write!(f, "Boolean"),
            Types::Counter => write!(f, "Counter"),
            Types::Date => write!(f, "Date"),
            Types::Decimal => write!(f, "Decimal"),
            Types::Double => write!(f, "Double"),
            Types::Duration => write!(f, "Duration"),
            Types::Float => write!(f, "Float"),
            Types::Inet => write!(f, "Inet"),
            Types::Int => write!(f, "Int"),
            Types::SmallInt => write!(f, "SmallInt"),
            Types::Text => write!(f, "Text"),
            Types::Time => write!(f, "Time"),
            Types::Timestamp => write!(f, "Timestamp"),
            Types::Timeuuid => write!(f, "Timeuuid"),
            Types::TinyInt => write!(f, "TinyInt"),
            Types::Uuid => write!(f, "Uuid"),
            Types::Varchar => write!(f, "Varchar"),
            Types::Varint => write!(f, "Varint"),
            Types::Map => write!(f, "Map"),
            Types::List => write!(f, "List"),
            Types::Set => write!(f, "Set"),
            Types::Tuple => write!(f, "Tuple"),
            Types::Frozen => write!(f, "Frozen"),
        }
    }
}

#[derive(FromAttributes, Clone)]
#[darling(attributes(charybdis))]
pub struct FieldAttributes {
    #[darling(default)]
    pub ignore: Option<bool>,
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
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
    pub fn from_field(field: &syn::Field, is_partition_key: bool, is_clustering_key: bool) -> Self {
        FieldAttributes::from_attributes(&field.attrs)
            .map(|char_attrs| {
                let ident = field.ident.clone().unwrap();
                return Field {
                    name: ident.to_string(),
                    ident: ident.clone(),
                    ty: field.ty.clone(),
                    ty_path: match &field.ty {
                        syn::Type::Path(type_path) => type_path.clone(),
                        _ => panic!("Only type path is supported!"),
                    },
                    char_attrs,
                    attrs: field.attrs.clone(),
                    span: field.span(),
                    is_partition_key,
                    is_clustering_key,
                };
            })
            .unwrap()
    }

    pub fn type_string(&self) -> String {
        self.ty.to_token_stream().to_string()
    }

    pub fn is_primary_key(&self) -> bool {
        self.is_partition_key || self.is_clustering_key
    }

    pub fn is_list(&self) -> bool {
        self.type_string().contains(Types::List.to_string().as_str())
    }

    pub fn is_set(&self) -> bool {
        self.type_string().contains(Types::Set.to_string().as_str())
    }

    pub fn is_counter(&self) -> bool {
        self.type_string().contains(Types::Counter.to_string().as_str())
    }
}

pub struct CharybdisFields {
    pub all_fields: Vec<Field>,
    pub partition_key_fields: Vec<Field>,
    pub clustering_key_fields: Vec<Field>,
    pub primary_key_fields: Vec<Field>,
    pub db_fields: Vec<Field>,
}

impl CharybdisFields {
    pub fn non_primary_key_db_fields(&self) -> Vec<Field> {
        self.db_fields
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
        let mut primary_key_fields = vec![];
        let mut db_fields = vec![];
        let mut all_fields = vec![];

        let partition_keys = args.partition_keys.clone().unwrap_or(vec![]);
        let clustering_keys = args.clustering_keys.clone().unwrap_or(vec![]);
        let primary_keys = partition_keys
            .clone()
            .iter()
            .chain(clustering_keys.clone().iter())
            .cloned()
            .collect::<Vec<String>>();

        for key in partition_keys {
            let field = named_fields
                .named
                .iter()
                .find(|f| f.ident.clone().unwrap().to_string() == key)
                .expect(&format!("Partition key {} not found in struct fields", key));

            let char_field = Field::from_field(field, true, false);

            partition_key_fields.push(char_field.clone());
            primary_key_fields.push(char_field.clone());
            all_fields.push(char_field.clone());
            db_fields.push(char_field.clone());
        }

        for key in clustering_keys {
            let field = named_fields
                .named
                .iter()
                .find(|f| f.ident.clone().unwrap().to_string() == key)
                .expect(&format!("Clustering key {} not found in struct fields", key));

            let char_field = Field::from_field(field, false, true);

            clustering_key_fields.push(char_field.clone());
            primary_key_fields.push(char_field.clone());
            all_fields.push(char_field.clone());
            db_fields.push(char_field.clone());
        }

        for field in &named_fields.named {
            let field_name = field.ident.clone().unwrap().to_string();
            if !primary_keys.contains(&field_name) {
                let char_field = Field::from_field(field, false, false);

                all_fields.push(char_field.clone());

                if !char_field.char_attrs.ignore.unwrap_or(false) {
                    db_fields.push(char_field.clone());
                }
            }
        }

        Self {
            partition_key_fields,
            clustering_key_fields,
            primary_key_fields,
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

    /// Map charybdis(ignore) to scylla(skip)
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
