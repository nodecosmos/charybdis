use std::collections::HashMap;

use proc_macro2::TokenStream;

enum HashSplitter {
    Line,
    Pair,
}

impl<'a> HashSplitter {
    const LINE: &'static str = ";";
    const PAIR: &'static str = "=>";

    fn split(&self, string: &'a str) -> Vec<&'a str> {
        let str = match self {
            HashSplitter::Line => string.split(Self::LINE),
            HashSplitter::Pair => string.split(Self::PAIR),
        };

        str.collect()
    }
}

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
    for pair in HashSplitter::Line.split(&hash) {
        let pair = pair.trim();
        let pair: Vec<&str> = HashSplitter::Pair.split(&pair);

        if pair.len() != 2 {
            continue;
        }

        let key = pair[0].trim_matches('\'').trim();
        let value = pair[1].trim_matches('\'');

        let token = syn::parse_str::<TokenStream>(value).unwrap();

        parsed_field_types_hash.insert(key.to_string(), token);
    }

    parsed_field_types_hash
}
