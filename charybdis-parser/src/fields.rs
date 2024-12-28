use std::collections::{HashMap, HashSet};

use darling::FromAttributes;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, FieldsNamed, GenericArgument, PathArguments, Type};

use crate::traits::CharybdisMacroArgs;

#[derive(Clone, PartialEq, strum_macros::Display, strum_macros::EnumString)]
pub enum CqlType {
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
    Ignored,
    /// This is used when the type is not recognized. In future we might want to extend recognition to UDTs,
    /// so we can panic if type is not recognized.
    Unknown,
}

#[derive(FromAttributes, Clone)]
#[darling(attributes(charybdis))]
pub struct FieldAttributes {
    #[darling(default)]
    pub ignore: Option<bool>,
}

pub struct Field<'a> {
    pub name: String,
    pub ident: syn::Ident,
    pub ty: Type,
    pub ty_path: syn::TypePath,
    pub outer_type: CqlType,
    pub span: proc_macro2::Span,
    pub attrs: &'a Vec<syn::Attribute>,
    pub ignore: bool,
    pub is_partition_key: bool,
    pub is_clustering_key: bool,
    pub is_static_column: bool,
}

impl<'a> Field<'a> {
    pub fn outer_type(ty: &Type, ignore: bool) -> CqlType {
        if ignore {
            return CqlType::Ignored;
        }

        if let Type::Path(type_path) = ty {
            // Handle the outer type being an Option
            if let Some(last_segment) = type_path.path.segments.last() {
                if last_segment.ident == "Option" {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(GenericArgument::Type(Type::Path(inner_type_path))) = args.args.first() {
                            let inner = inner_type_path.path.segments.last().expect("No inner type found");

                            return inner
                                .ident
                                .to_string()
                                .parse::<CqlType>()
                                .ok()
                                .unwrap_or(CqlType::Unknown);
                        }
                    }
                } else {
                    return last_segment
                        .ident
                        .to_string()
                        .parse::<CqlType>()
                        .ok()
                        .unwrap_or(CqlType::Unknown);
                }
            }

            panic!("Unable to parse type for field: {:?}", ty);
        }

        panic!("Unable to parse type for field: {:?}", ty);
    }

    pub fn from_field(
        field: &'a syn::Field,
        is_partition_key: bool,
        is_clustering_key: bool,
        is_static_column: bool,
    ) -> Self {
        FieldAttributes::from_attributes(&field.attrs)
            .map(|char_attrs| {
                let ignore = char_attrs.ignore.unwrap_or(false);
                let ident = field.ident.clone().unwrap();

                Field {
                    name: ident.to_string(),
                    ident,
                    ty: field.ty.clone(),
                    ty_path: match &field.ty {
                        Type::Path(type_path) => type_path.clone(),
                        _ => panic!("Only type path is supported!"),
                    },
                    outer_type: Field::outer_type(&field.ty, ignore),
                    span: field.span(),
                    attrs: &field.attrs,
                    ignore,
                    is_partition_key,
                    is_clustering_key,
                    is_static_column,
                }
            })
            .unwrap()
    }

    pub fn is_primary_key(&self) -> bool {
        self.is_partition_key || self.is_clustering_key
    }

    pub fn is_list(&self) -> bool {
        self.outer_type == CqlType::List
    }

    pub fn is_set(&self) -> bool {
        self.outer_type == CqlType::Set
    }

    pub fn is_map(&self) -> bool {
        self.outer_type == CqlType::Map
    }

    pub fn is_collection(&self) -> bool {
        self.is_list() || self.is_set() || self.is_map()
    }

    pub fn is_counter(&self) -> bool {
        self.outer_type == CqlType::Counter
    }

    pub fn is_tuple(&self) -> bool {
        self.outer_type == CqlType::Tuple
    }

    pub fn is_frozen(&self) -> bool {
        self.ty_path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Frozen")
    }
}

#[derive(Default)]
pub struct CharybdisFields<'a> {
    pub all_fields: Vec<Field<'a>>,
    pub partition_key_fields: Vec<&'a Field<'a>>,
    pub clustering_key_fields: Vec<&'a Field<'a>>,
    pub primary_key_fields: Vec<&'a Field<'a>>,
    pub db_fields: Vec<&'a Field<'a>>,
    pub global_secondary_index_fields: Vec<&'a Field<'a>>,
    pub local_secondary_index_fields: Vec<&'a Field<'a>>,
}

impl CharybdisFields<'_> {
    pub fn non_primary_key_db_fields(&self) -> Vec<&Field> {
        self.db_fields
            .iter()
            .filter(|field| !field.is_primary_key())
            .cloned()
            .collect()
    }

    pub fn non_db_fields(&self) -> Vec<&Field> {
        self.all_fields.iter().filter(|field| field.ignore).collect()
    }
}

impl<'a> CharybdisFields<'a> {
    fn new(named_fields: &'a FieldsNamed, args: &CharybdisMacroArgs) -> Self {
        let mut me = Self::default();

        for field in &named_fields.named {
            let field_name = field.ident.clone().expect("field must have an identifier").to_string();
            let is_partition_key = args.partition_keys().contains(&field_name);
            let is_clustering_key = args.clustering_keys().contains(&field_name);
            let is_static_column = args.static_columns().contains(&field_name);

            let ch_field = Field::from_field(field, is_partition_key, is_clustering_key, is_static_column);

            if ch_field.is_tuple() && !ch_field.is_frozen() {
                panic!("Tuple field {} must be frozen. Use Frozen<Tuple<T>>.", field_name);
            }

            if is_partition_key && is_clustering_key {
                panic!("Field {} cannot be both partition and clustering key", field_name);
            }

            if is_static_column && (is_partition_key || is_clustering_key) {
                panic!(
                    "Field {} cannot be both static column and partition or clustering key",
                    field_name
                );
            }

            me.all_fields.push(ch_field);
        }

        me
    }

    pub fn populate(&'a mut self, args: &CharybdisMacroArgs) -> &'a Self {
        let mut partition_key_fields: Vec<Option<&Field>> = vec![None; args.partition_keys().len()];
        let mut clustering_key_fields: Vec<Option<&Field>> = vec![None; args.clustering_keys().len()];
        let mut primary_key_fields: Vec<Option<&Field>> =
            vec![None; args.partition_keys().len() + args.clustering_keys().len()];

        // primary key indexes by name
        let partition_key_indexes_by_name = args
            .partition_keys()
            .iter()
            .enumerate()
            .map(|(i, key)| (key, i))
            .collect::<HashMap<&String, usize>>();

        let clustering_key_indexes_by_name = args
            .clustering_keys()
            .iter()
            .enumerate()
            .map(|(i, key)| (key, i))
            .collect::<HashMap<&String, usize>>();

        #[allow(suspicious_double_ref_op)]
        let primary_key_indexes_by_name = args
            .primary_key()
            .iter()
            .enumerate()
            .map(|(i, key)| (key.clone(), i))
            .collect::<HashMap<&String, usize>>();

        // used to validate that provided macro args are present in struct fields
        let mut pk_struct_fields = HashSet::new();
        let mut ck_struct_fields = HashSet::new();
        let mut static_struct_fields = HashSet::new();

        // populate fields
        for ch_field in self.all_fields.iter() {
            if !ch_field.ignore {
                self.db_fields.push(ch_field);
            }

            if ch_field.is_static_column {
                static_struct_fields.insert(ch_field.name.clone());
            }

            if args.global_secondary_indexes().contains(&ch_field.name) {
                self.global_secondary_index_fields.push(ch_field);
            }

            if args.local_secondary_indexes().contains(&ch_field.name) {
                self.local_secondary_index_fields.push(ch_field);
            }

            if ch_field.is_partition_key {
                let partition_key_index = *partition_key_indexes_by_name
                    .get(&ch_field.name)
                    .expect("index must be set");
                partition_key_fields[partition_key_index] = Some(ch_field);

                let primary_key_index = *primary_key_indexes_by_name
                    .get(&ch_field.name)
                    .expect("index must be set");
                primary_key_fields[primary_key_index] = Some(ch_field);

                pk_struct_fields.insert(ch_field.name.clone());
            } else if ch_field.is_clustering_key {
                let clustering_key_index = *clustering_key_indexes_by_name
                    .get(&ch_field.name)
                    .expect("index must be set");
                clustering_key_fields[clustering_key_index] = Some(ch_field);

                let primary_key_index = *primary_key_indexes_by_name
                    .get(&ch_field.name)
                    .expect("index must be set");
                primary_key_fields[primary_key_index] = Some(ch_field);

                ck_struct_fields.insert(ch_field.name.clone());
            }
        }

        // validate that provided macro args are present in struct fields
        for key in args.partition_keys() {
            if !pk_struct_fields.contains(key) {
                panic!("Partition key {} not found in struct fields", key);
            }
        }

        for key in args.clustering_keys() {
            if !ck_struct_fields.contains(key) {
                panic!("Clustering key {} not found in struct fields", key);
            }
        }

        for key in args.static_columns() {
            if !static_struct_fields.contains(key) {
                panic!("Static column {} not found in struct fields", key);
            }
        }

        // populate primary key fields
        self.partition_key_fields = partition_key_fields.into_iter().flatten().collect();
        self.clustering_key_fields = clustering_key_fields.into_iter().flatten().collect();
        self.primary_key_fields = primary_key_fields.into_iter().flatten().collect();

        self
    }

    pub(crate) fn db_fields(named_fields: &FieldsNamed) -> Vec<Field> {
        let mut db_fields = vec![];

        for field in &named_fields.named {
            let field = Field::from_field(field, false, false, false);

            if !field.ignore {
                db_fields.push(field);
            }
        }

        db_fields
    }

    pub fn from_input(input: &'a DeriveInput, args: &'a CharybdisMacroArgs) -> Self {
        match &input.data {
            Data::Struct(data) => match &data.fields {
                Fields::Named(named_fields) => CharybdisFields::new(named_fields, args),
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
