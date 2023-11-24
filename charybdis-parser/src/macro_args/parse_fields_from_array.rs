use crate::schema::secondary_indexes::LocalIndexTarget;
use colored::*;
use quote::ToTokens;
use syn::{Expr, ExprArray, ExprTuple};

pub(crate) fn parse_arr_expr_from_literals(array_expr: ExprArray) -> Vec<String> {
    array_expr
        .elems
        .into_iter()
        .map(|elem| elem.to_token_stream().to_string())
        .collect()
}

pub(crate) fn parse_loc_sec_idx_array_expr(array_expr: ExprArray) -> Vec<LocalIndexTarget> {
    array_expr
        .elems
        .into_iter()
        .filter_map(|elem| {
            if let Expr::Tuple(ExprTuple { elems, .. }) = elem {
                let elems: Vec<_> = elems.iter().collect();

                if elems.len() != 2 {
                    panic!(
                        "{} {}",
                        "Expected 2 elements in local_secondary_indexes tuple, found {}".bright_red(),
                        elems.len()
                    );
                }

                let pk = if let Expr::Array(ref array) = *elems[0] {
                    parse_arr_expr_from_literals(array.clone())
                } else {
                    return None;
                };

                let ck = if let Expr::Array(ref array) = *elems[1] {
                    parse_arr_expr_from_literals(array.clone())
                } else {
                    return None;
                };

                Some(LocalIndexTarget { pk, ck })
            } else {
                panic!(
                    "{}\n{}\n{} {}",
                    "Incorrect local secondary index format.".bright_red(),
                    "Expected an array of tuples.".bright_red(),
                    "Found:".bright_red(),
                    elem.to_token_stream().to_string().bright_yellow()
                );
            }
        })
        .collect()
}
