use crate::traits::fields::{FieldsFindFirstFns, FieldsFindFn, FieldsQuery};
use charybdis_parser::fields::{CharybdisFields, Field};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;

const MAX_FIND_BY_FIELDS: usize = 3;

/// for up to 3 primary keys, generate find_by_primary_key functions e.g.
/// ```rust ignore
///   #[charybdis_model(
///         table_name = posts,
///         partition_keys = [date],
///         clustering_keys = [category_id, title],
///         global_secondary_indexes = []
///     )]
///    pub struct Post {...}
/// ```
/// We will generate the following functions:
/// ```rust ignore
///   Post::find_by_date(date: Date)
///   Post::find_by_date_and_category_id(date: Date, category_id: Uuid)
///   Post::find_by_date_and_category_id_and_title(date: Date, category_id: Uuid, title: Text)
/// ```
pub(crate) fn find_by_primary_keys_functions(
    struct_name: &syn::Ident,
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let table_name = ch_args.table_name();
    let comma_sep_cols = fields.db_fields.comma_sep_cols();

    let primary_key_stack = &fields.primary_key_fields;
    let mut generated = quote! {};

    for i in 0..primary_key_stack.len() {
        if i == MAX_FIND_BY_FIELDS {
            break;
        }

        let current_fields = primary_key_stack.iter().take(i + 1).cloned().collect::<Vec<Field>>();
        let query_str = format!(
            "SELECT {} FROM {} WHERE {}",
            comma_sep_cols,
            table_name,
            current_fields.where_placeholders()
        );

        if current_fields.len() == primary_key_stack.len() {
            // for complete primary key we get single row
            generated.extend(current_fields.find_one_fn(struct_name, &query_str));
        } else {
            // for partial primary key we get a stream
            generated.extend(current_fields.find_fn(struct_name, &query_str));
        }

        // query one row
        generated.extend(current_fields.find_first_fn(struct_name, &query_str));
        generated.extend(current_fields.maybe_find_first_fn(struct_name, &query_str));
    }

    generated
}

pub(crate) fn find_by_local_secondary_index(
    struct_name: &syn::Ident,
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let table_name = ch_args.table_name();
    let comma_sep_cols = fields.db_fields.comma_sep_cols();
    let partition_keys = &fields.partition_key_fields;
    let lsi_fields = &fields.local_secondary_index_fields;

    let mut generated = quote! {};

    lsi_fields.iter().for_each(|lsi| {
        let mut current_fields = partition_keys.clone();
        current_fields.push(lsi.clone());

        let query_str = format!(
            "SELECT {} FROM {} WHERE {}",
            comma_sep_cols,
            table_name,
            current_fields.where_placeholders()
        );
        let find_fn = current_fields.find_fn(struct_name, &query_str);
        let find_first_fn = current_fields.find_first_fn(struct_name, &query_str);
        let maybe_find_first_fn = current_fields.maybe_find_first_fn(struct_name, &query_str);

        generated.extend(find_fn);
        generated.extend(find_first_fn);
        generated.extend(maybe_find_first_fn);
    });

    generated
}
