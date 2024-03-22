use crate::traits::fields::FieldsNames;
use charybdis_parser::fields::Field;

pub(crate) trait FieldsQuery {
    fn comma_sep_cols(&self) -> String;
    fn insert_bind_markers(&self) -> String;
    fn set_bind_markers(&self) -> String;
    fn where_placeholders(&self) -> String;
    fn where_bind_markers(&self) -> String;
}

impl FieldsQuery for Vec<Field> {
    fn comma_sep_cols(&self) -> String {
        self.names().join(", ")
    }

    fn insert_bind_markers(&self) -> String {
        let str_vec = self
            .iter()
            .map(|field| format!(":{}", field.name))
            .collect::<Vec<String>>()
            .join(", ");

        return str_vec;
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
