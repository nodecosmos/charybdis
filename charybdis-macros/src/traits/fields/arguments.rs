use crate::traits::r#type::TypeWithoutOptions;
use charybdis_parser::fields::Field;
use syn::parse_str;

pub(crate) trait FieldsToArguments {
    fn to_fn_args(&self) -> Vec<syn::FnArg>;
}

impl FieldsToArguments for Vec<Field> {
    fn to_fn_args(&self) -> Vec<syn::FnArg> {
        self.iter()
            .map(|field| {
                let type_wo_options = field.ty.type_without_options();
                parse_str::<syn::FnArg>(&format!("{}: {}", field.name, type_wo_options)).unwrap()
            })
            .collect::<Vec<syn::FnArg>>()
    }
}
