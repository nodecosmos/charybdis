use charybdis_parser::fields::Field;

pub(crate) fn where_placeholders(fields: &Vec<Field>) -> String {
    fields
        .iter()
        .map(|field| format!("{} = ?", field.name))
        .collect::<Vec<String>>()
        .join(" AND ")
}

pub(crate) fn where_bind_markers(fields: &Vec<Field>) -> String {
    fields
        .iter()
        .map(|field| format!("{} = :{}", field.name, field.name))
        .collect::<Vec<String>>()
        .join(" AND ")
}

pub(crate) fn insert_bind_markers(fields: &Vec<Field>) -> String {
    let str_vec = fields
        .iter()
        .map(|field| format!(":{}", field.name))
        .collect::<Vec<String>>()
        .join(", ");

    return str_vec;
}

pub(crate) fn set_bind_markers(fields: Vec<Field>) -> String {
    fields
        .iter()
        .map(|field| format!("{} = :{}", field.name, field.name))
        .collect::<Vec<String>>()
        .join(", ")
}
