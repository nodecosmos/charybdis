use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn primary_key_const(ch_args: &CharybdisMacroArgs) -> ImplItem {
    let mut primary_key = ch_args.partition_keys.clone().unwrap();
    let mut clustering_keys = ch_args.clustering_keys.clone().unwrap();

    primary_key.append(clustering_keys.as_mut());

    let generated = quote! {
        const PRIMARY_KEY: &'static [&'static str] = &[#(#primary_key),*];
    };

    syn::parse_quote!(#generated)
}
