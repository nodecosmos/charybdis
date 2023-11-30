use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ImplItem;

pub(crate) fn partition_keys_const(ch_args: &CharybdisMacroArgs) -> ImplItem {
    let partition_keys = ch_args.partition_keys.as_ref().unwrap_or_else(|| {
        panic!(
            r#"
                The `partition_keys` attribute is required for the `charybdis_model` macro.
                Please provide a list of partition keys for the model.
                e.g. #[charybdis_model(partition_keys = ["id"])]
            "#
        )
    });

    if partition_keys.is_empty() {
        panic!(
            r#"
                The `partition_keys` attribute must define at least one partition key.
            "#
        )
    }

    let partition_keys = partition_keys.iter().map(|pk| quote!(#pk));

    let generated: TokenStream = quote! {
        const PARTITION_KEYS: &'static [&'static str] = &[#(#partition_keys),*];
    };

    syn::parse_quote!(#generated)
}
