use proc_macro2::TokenStream;
use syn::parse_str;

pub(crate) fn serialized_field_value_adder(fields: Vec<String>) -> TokenStream {
    let fields_str: String = fields
        .iter()
        .map(|key| format!("serialized.add_value(&self.{})?;", key))
        .collect::<Vec<String>>()
        .join("\n");

    parse_str(&fields_str).unwrap()
}

pub(crate) fn serialized_value_adder(values: Vec<String>) -> TokenStream {
    let values_str: String = values
        .iter()
        .map(|key| format!("serialized.add_value(&{})?;", key))
        .collect::<Vec<String>>()
        .join("\n");

    parse_str(&values_str).unwrap()
}
