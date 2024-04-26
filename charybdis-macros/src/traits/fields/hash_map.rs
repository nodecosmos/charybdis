use quote::quote;
use syn::Attribute;

use charybdis_parser::fields::Field;

/// It is used by `partial_model_macro_generator` to associate field **types**
/// and field **attributes** with field names for the generated partial model.
pub(crate) trait FieldHashMapString {
    /// key is field name and value is field type.
    fn field_types_hash_map_string(&self) -> String;

    /// key is field name and value is field attributes.
    fn field_attributes_hash_map_string(&self) -> String;
}

impl FieldHashMapString for Vec<Field<'_>> {
    fn field_types_hash_map_string(&self) -> String {
        let mut field_types = quote! {};

        for field in self.iter() {
            let field_ident = &field.ident;
            let ty = &field.ty;

            field_types.extend(quote! { #field_ident => #ty; });
        }

        field_types.to_string().replace('\n', "")
    }

    fn field_attributes_hash_map_string(&self) -> String {
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
