use charybdis_parser::fields::Field;

use crate::traits::fields::FieldsNames;

pub(crate) trait FieldsQuery {
    fn comma_sep_cols(&self) -> String;
    fn insert_bind_markers(&self) -> String;
    fn set_bind_markers(&self) -> String;
    fn where_placeholders(&self) -> String;
    fn where_bind_markers(&self) -> String;
}

impl FieldsQuery for Vec<&Field<'_>> {
    fn comma_sep_cols(&self) -> String {
        self.names()
            .iter()
            .map(|s| format!(r#""{s}""#))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn insert_bind_markers(&self) -> String {
        let str_vec = self
            .iter()
            .map(|field| format!(":{}", field.name))
            .collect::<Vec<String>>()
            .join(", ");

        str_vec
    }

    fn set_bind_markers(&self) -> String {
        self.iter()
            .map(|field| format!("{} = :{}", field.name, field.name))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn where_placeholders(&self) -> String {
        self.iter()
            .map(|field| format!("{} = ?", field.name))
            .collect::<Vec<String>>()
            .join(" AND ")
    }

    fn where_bind_markers(&self) -> String {
        self.iter()
            .map(|field| format!("{} = :{}", field.name, field.name))
            .collect::<Vec<String>>()
            .join(" AND ")
    }
}
