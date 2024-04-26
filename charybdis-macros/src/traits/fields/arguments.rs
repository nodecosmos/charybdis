use syn::parse_str;

use charybdis_parser::fields::Field;

use crate::traits::r#type::TypeWithoutOptions;

pub(crate) trait FieldsToArguments {
    fn to_fn_args(&self) -> Vec<syn::FnArg>;
}

impl FieldsToArguments for Field<'_> {
    fn to_fn_args(&self) -> Vec<syn::FnArg> {
        let type_wo_options = self.ty.type_without_options();
        let res = parse_str::<syn::FnArg>(&format!("{}: {}", self.name, type_wo_options)).unwrap();

        vec![res]
    }
}

impl FieldsToArguments for Vec<&Field<'_>> {
    fn to_fn_args(&self) -> Vec<syn::FnArg> {
        self.iter()
            .map(|field| {
                let type_wo_options = field.ty.type_without_options();
                parse_str::<syn::FnArg>(&format!("{}: {}", field.name, type_wo_options)).unwrap()
            })
            .collect::<Vec<syn::FnArg>>()
    }
}
