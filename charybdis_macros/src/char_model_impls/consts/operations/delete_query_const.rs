use charybdis_parser::macro_args::CharybdisMacroArgs;
use quote::quote;
use syn::ImplItem;

pub(crate) fn delete_query_const(ch_args: &CharybdisMacroArgs) -> ImplItem {
    let mut primary_key: Vec<String> = ch_args.partition_keys.clone().unwrap();
    let mut clustering_keys: Vec<String> = ch_args.clustering_keys.clone().unwrap();

    primary_key.append(clustering_keys.as_mut());

    let table_name: &String = ch_args.table_name.as_ref().unwrap();
    let primary_key_where_clause: String = primary_key.join(" = ? AND ");

    let query_str: String = format!("DELETE FROM {} WHERE {} = ?", table_name, primary_key_where_clause,);

    let generated = quote! {
        const DELETE_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}

pub(crate) fn delete_by_partition_key_query_const(ch_args: &CharybdisMacroArgs) -> ImplItem {
    let partition_key: Vec<String> = ch_args.partition_keys.clone().unwrap();
    let table_name: &String = ch_args.table_name.as_ref().unwrap();
    let partition_key_where_clause: String = partition_key.join(" = ? AND ");

    let query_str: String = format!("DELETE FROM {} WHERE {} = ?", table_name, partition_key_where_clause,);

    let generated = quote! {
        const DELETE_BY_PARTITION_KEY_QUERY: &'static str = #query_str;
    };

    syn::parse_quote!(#generated)
}
