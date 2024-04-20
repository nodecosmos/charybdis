use charybdis_parser::fields::Field;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) trait FieldsAsTuple {
    fn types(&self) -> Vec<syn::Type>;
    fn values(&self) -> Vec<TokenStream>;
}

impl FieldsAsTuple for Vec<&Field<'_>> {
    fn types(&self) -> Vec<syn::Type> {
        self.iter().map(|field| field.ty.clone()).collect()
    }

    fn values(&self) -> Vec<TokenStream> {
        self.iter()
            .map(|field| {
                let field_name = field.ident.clone();
                quote! { self.#field_name.clone() }
            })
            .collect::<Vec<_>>()
    }
}

impl FieldsAsTuple for Vec<syn::FnArg> {
    fn types(&self) -> Vec<syn::Type> {
        self.iter()
            .map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    pat_type.ty.as_ref().clone()
                } else {
                    panic!("Expected typed argument")
                }
            })
            .collect()
    }

    fn values(&self) -> Vec<TokenStream> {
        self.iter()
            .map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    let pat = pat_type.pat.clone();
                    quote! { #pat }
                } else {
                    panic!("Expected typed argument")
                }
            })
            .collect()
    }
}

pub(crate) trait Tuple {
    fn types_tp(&self) -> TokenStream;
    fn values_tp(&self) -> TokenStream;
}

impl<T: FieldsAsTuple> Tuple for T {
    fn types_tp(&self) -> TokenStream {
        let types = self.types();

        return if types.len() == 1 {
            let single_type = types.first().unwrap();
            quote! {
                (#single_type,)
            }
        } else {
            quote! {
                (#(#types),*)
            }
        };
    }

    fn values_tp(&self) -> TokenStream {
        let values = self.values();

        return if values.len() == 1 {
            let single_value = values.first().unwrap();
            quote! {
                (#single_value,)
            }
        } else {
            quote! {
                (#(#values),*)
            }
        };
    }
}
