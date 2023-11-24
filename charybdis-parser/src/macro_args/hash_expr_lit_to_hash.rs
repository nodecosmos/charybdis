use proc_macro2::TokenStream;
use std::collections::HashMap;

pub(crate) fn hash_expr_lit_to_hash(expr: syn::Expr, cha_attr_name: String) -> HashMap<String, TokenStream> {
    // parse ruby style hash
    let hash = match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => lit_str.value(),
        _ => panic!("{} must be a string", cha_attr_name),
    };

    // hashmap
    let mut parsed_field_types_hash = HashMap::new();
    for pair in hash.split(';') {
        let pair = pair.trim();
        let pair: Vec<&str> = pair.split("=>").collect();

        if pair.len() != 2 {
            continue;
        }

        let key = pair[0].trim_matches('\'').trim();
        let value = pair[1].trim_matches('\'');

        // println!("key: {}", key);
        // println!("value: {}", value);

        let token = syn::parse_str::<TokenStream>(value).unwrap();

        parsed_field_types_hash.insert(key.to_string(), token);
    }

    parsed_field_types_hash
}
