use proc_macro2::TokenStream;
use syn::{GenericArgument, PathArguments, Type};

pub fn type_without_options(o_type: &Type) -> TokenStream {
    let mut type_name = quote::quote! { #o_type };

    match o_type {
        Type::Path(type_path) => {
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
        _ => {}
    }

    type_name
}
