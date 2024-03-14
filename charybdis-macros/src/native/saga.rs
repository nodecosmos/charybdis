use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn append_saga_field(input: &mut DeriveInput, args: &CharybdisMacroArgs) {
    if args.saga.unwrap_or(false) {
        let saga_field = syn::parse_quote! {
            #[charybdis(ignore)]
            pub saga: Option<charybdis::saga::Saga<Self>>
        };

        if let syn::Data::Struct(data_struct) = &mut input.data {
            if let syn::Fields::Named(fields_named) = &mut data_struct.fields {
                fields_named.named.push(saga_field);
            }
        }
    }
}

pub(crate) fn saga_method(args: &CharybdisMacroArgs) -> Option<proc_macro2::TokenStream> {
    if args.saga.unwrap_or(false) {
        Some(quote! {
            pub fn saga(&mut self) -> &mut charybdis::saga::Saga<Self> {
                self._saga.get_or_insert_with(|| charybdis::saga::Saga::new())
            }
        })
    } else {
        None
    }
}
