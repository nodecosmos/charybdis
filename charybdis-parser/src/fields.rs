use crate::macro_args::CharybdisMacroArgs;
use darling::FromAttributes;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, FieldsNamed, GenericArgument, PathArguments, Type};

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
    Frozen,
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

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub ident: syn::Ident,
    pub ty: Type,
    pub ty_path: syn::TypePath,
    pub outer_type: CqlType,
    pub char_attrs: FieldAttributes,
    pub attrs: Vec<syn::Attribute>,
    pub span: proc_macro2::Span,
    pub is_partition_key: bool,
    pub is_clustering_key: bool,
}

impl Field {
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
                                .unwrap_or_else(|| CqlType::Unknown);
                        }
                    }
                } else {
                    return last_segment
                        .ident
                        .to_string()
                        .parse::<CqlType>()
                        .ok()
                        .unwrap_or_else(|| CqlType::Unknown);
                }
            }

            panic!("Unable to parse type for field: {:?}", ty);
        }

        panic!("Unable to parse type for field: {:?}", ty);
    }

    pub fn from_field(field: &syn::Field, is_partition_key: bool, is_clustering_key: bool) -> Self {
        FieldAttributes::from_attributes(&field.attrs)
            .map(|char_attrs| {
                let ignore = char_attrs.ignore.unwrap_or(false);
                let ident = field.ident.clone().unwrap();
                return Field {
                    name: ident.to_string(),
                    ident: ident.clone(),
                    ty: field.ty.clone(),
                    ty_path: match &field.ty {
                        Type::Path(type_path) => type_path.clone(),
                        _ => panic!("Only type path is supported!"),
                    },
                    outer_type: Field::outer_type(&field.ty, ignore),
                    char_attrs,
                    attrs: field.attrs.clone(),
                    span: field.span(),
                    is_partition_key,
                    is_clustering_key,
                };
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
}

#[derive(Default)]
pub struct CharybdisFields {
    pub all_fields: Vec<Field>,
    pub partition_key_fields: Vec<Field>,
    pub clustering_key_fields: Vec<Field>,
    pub primary_key_fields: Vec<Field>,
    pub db_fields: Vec<Field>,
    pub global_secondary_index_fields: Vec<Field>,
    pub local_secondary_index_fields: Vec<Field>,
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
        let mut global_secondary_index_fields = vec![];
        let mut local_secondary_index_fields = vec![];

        let partition_keys = args.partition_keys();
        let clustering_keys = args.clustering_keys();
        let global_secondary_indexes = args.global_secondary_indexes();
        let local_secondary_indexes = args.local_secondary_indexes();

        // make sure that all partition_keys are present in the struct fields
        for key in &partition_keys {
            named_fields
                .named
                .iter()
                .find(|f| f.ident.as_ref().is_some_and(|i| &i.to_string() == key))
                .expect(&format!("Partition key {} not found in struct fields", key));
        }

        // make sure that all clustering_keys are present in the struct fields
        for key in &clustering_keys {
            named_fields
                .named
                .iter()
                .find(|f| f.ident.as_ref().is_some_and(|i| &i.to_string() == key))
                .expect(&format!("Clustering key {} not found in struct fields", key));
        }

        for field in &named_fields.named {
            let field_name = field.ident.clone().unwrap().to_string();
            let is_partition_key = partition_keys.contains(&field_name);
            let is_clustering_key = clustering_keys.contains(&field_name);
            let ch_field = Field::from_field(field, is_partition_key, is_clustering_key);

            if is_partition_key && is_clustering_key {
                panic!("Field {} cannot be both partition and clustering key", field_name);
            }

            all_fields.push(ch_field.clone());

            if !ch_field.char_attrs.ignore.unwrap_or(false) {
                db_fields.push(ch_field.clone());
            }

            if global_secondary_indexes.contains(&field_name) {
                global_secondary_index_fields.push(ch_field.clone());
            }

            if local_secondary_indexes.contains(&field_name) {
                local_secondary_index_fields.push(ch_field.clone());
            }

            if is_partition_key {
                partition_key_fields.push(ch_field.clone());
                primary_key_fields.push(ch_field);
            } else if is_clustering_key {
                clustering_key_fields.push(ch_field.clone());
                primary_key_fields.push(ch_field);
            }
        }

        Self {
            partition_key_fields,
            clustering_key_fields,
            primary_key_fields,
            all_fields,
            db_fields,
            global_secondary_index_fields,
            local_secondary_index_fields,
        }
    }

    pub(crate) fn db_fields(named_fields: &FieldsNamed) -> Vec<Field> {
        let mut all_fields = vec![];
        let mut db_fields = vec![];

        for field in &named_fields.named {
            let field = Field::from_field(field, false, false);

            all_fields.push(field.clone());

            if !field.char_attrs.ignore.unwrap_or(false) {
                db_fields.push(field.clone());
            }
        }

        db_fields
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
