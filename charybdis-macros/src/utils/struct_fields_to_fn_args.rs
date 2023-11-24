use crate::utils::type_without_options;
use syn::{parse_str, Field};

pub(crate) fn struct_fields_to_fn_args(
    struct_name: String,
    struct_fields: Vec<Field>,
    args: Vec<String>,
) -> Vec<syn::FnArg> {
    args.iter()
        .map(|key| {
            let key_type = struct_fields
                .iter()
                .find(|field| field.ident.as_ref().unwrap() == key)
                .unwrap_or_else(|| {
                    panic!(
                        "Key {} not found in struct {}. Partial models need to have complete primary key!",
                        key, struct_name
                    )
                })
                .ty
                .clone();

            let type_wo_options = type_without_options(&key_type);
            parse_str::<syn::FnArg>(&format!("{}: {}", key, type_wo_options)).unwrap()
        })
        .collect::<Vec<syn::FnArg>>()
}
