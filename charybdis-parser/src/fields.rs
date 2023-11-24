use syn::{Data, DeriveInput, Field, Fields, FieldsNamed};

pub struct CharybdisFields<'a> {
    named_fields: &'a FieldsNamed,
}

impl<'a> CharybdisFields<'a> {
    pub fn new(named_fields: &'a FieldsNamed) -> Self {
        Self { named_fields }
    }

    pub fn from_input(input: &'a DeriveInput) -> Self {
        match &input.data {
            Data::Struct(data) => match &data.fields {
                Fields::Named(named_fields) => Self { named_fields },
                _ => panic!("#[charybdis_model] works only for structs with named fields!"),
            },
            _ => panic!("#[charybdis_model] works only on structs!"),
        }
    }

    pub fn all_fields(&self) -> Vec<Field> {
        self.named_fields.named.iter().cloned().collect()
    }

    pub fn db_fields(&self) -> Vec<Field> {
        self.all_fields()
            .iter()
            .filter(|field| !has_ignore_attribute(field))
            .cloned()
            .collect()
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

// check if it has #[charybdis(ignore)]
fn has_ignore_attribute(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        if attr.path().is_ident("charybdis") {
            let maybe_ident: Result<syn::Ident, _> = attr.parse_args();
            if let Ok(ident) = maybe_ident {
                return ident == "ignore";
            }
        }
        false
    })
}
