use charybdis_parser::fields::Field;

pub(crate) fn comma_sep_cols(fields: &Vec<Field>) -> String {
    fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<Vec<String>>()
        .join(", ")
}
