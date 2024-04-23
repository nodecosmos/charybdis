use proc_macro2::Span;

pub trait ToIdent {
    fn to_ident(&self) -> syn::Ident;
}

impl ToIdent for String {
    fn to_ident(&self) -> syn::Ident {
        syn::Ident::new(self, Span::call_site())
    }
}
