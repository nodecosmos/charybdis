use charybdis_parser::fields::Field;

pub fn where_placeholders(fields: &Vec<String>) -> String {
    fields
        .iter()
        .map(|field| format!("{} = ?", field))
        .collect::<Vec<String>>()
        .join(" AND ")
}

pub fn where_bind_markers(fields: Vec<&Field>) -> String {
    fields
        .iter()
        .map(|field| format!("{} = :{}", field.ident, field.ident))
        .collect::<Vec<String>>()
        .join(" AND ")
}

pub fn insert_bind_markers(fields: &Vec<Field>) -> String {
    let str_vec = fields
        .iter()
        .map(|field| format!(":{}", field.ident))
        .collect::<Vec<String>>()
        .join(", ");

    return str_vec;
}

pub fn set_bind_markers(fields: Vec<&Field>) -> String {
    fields
        .iter()
        .map(|field| format!("{} = :{}", field.ident, field.ident))
        .collect::<Vec<String>>()
        .join(", ")
}
