use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn clustering_keys_const(ch_args: &CharybdisMacroArgs) -> ImplItem {
    let clustering_keys = ch_args.clustering_keys.clone().unwrap_or(vec![]);

    let generated = quote! {
        const CLUSTERING_KEYS:  &'static [&'static str] = &[#(#clustering_keys),*];
    };

    syn::parse_quote!(#generated)
}
