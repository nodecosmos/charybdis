use proc_macro2::TokenStream;
use syn::{GenericArgument, PathArguments, Type};

pub(crate) trait TypeWithoutOptions {
    fn type_without_options(&self) -> TokenStream;
}

impl TypeWithoutOptions for Type {
    fn type_without_options(&self) -> TokenStream {
        let mut type_name = quote::quote! { #self };

        if let Type::Path(type_path) = self {
            let first_segment = &type_path.path.segments[0];
            if first_segment.ident == "Option" {
                if let PathArguments::AngleBracketed(angle_bracketed_args) = &first_segment.arguments {
                    if let Some(GenericArgument::Type(inner_type)) = angle_bracketed_args.args.first() {
                        // Return the inner type of Option<T>
                        type_name = quote::quote! { #inner_type };
                    }
                }
            }
        }

        type_name
    }
}
