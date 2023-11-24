use syn::Field;

pub(crate) fn comma_sep_cols(fields: &Vec<Field>) -> String {
    fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect::<Vec<String>>()
        .join(", ")
}
