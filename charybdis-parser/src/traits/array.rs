use quote::ToTokens;
use std::collections::HashSet;
use syn::ExprArray;

pub(crate) trait ToStringCollection {
    fn to_vec(self) -> Vec<String>;
    fn to_set(self) -> HashSet<String>;
}

impl ToStringCollection for ExprArray {
    fn to_vec(self) -> Vec<String> {
        self.elems
            .into_iter()
            .map(|elem| elem.to_token_stream().to_string())
            .collect()
    }

    fn to_set(self) -> HashSet<String> {
        self.elems
            .into_iter()
            .map(|elem| elem.to_token_stream().to_string())
            .collect()
    }
}
