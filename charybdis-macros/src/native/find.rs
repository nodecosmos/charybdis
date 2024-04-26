use proc_macro2::TokenStream;
use quote::quote;

use charybdis_parser::fields::{CharybdisFields, Field};
use charybdis_parser::traits::CharybdisMacroArgs;

use crate::traits::fields::{FieldsFindFirstFns, FieldsFindFn, FieldsQuery};

const MAX_FIND_BY_FIELDS: usize = 3;

/// for up to 3 primary keys, generate find_by_primary_key functions e.g.
/// ```rust
/// use scylla::CachingSession;
/// use charybdis::errors::CharybdisError;
/// use charybdis::macros::charybdis_model;
/// use charybdis::stream::CharybdisModelStream;
/// use charybdis::types::{Date, Text, Uuid};
/// #[charybdis_model(
///         table_name = posts,
///         partition_keys = [date],
///         clustering_keys = [category_id, title],
///         global_secondary_indexes = [category_id],
///         local_secondary_indexes = [title]
/// )]
/// pub struct Post {
///     pub date: Date,
///     pub category_id: Uuid,
///     pub title: Text,
/// }
///
/// impl Post {
///     async fn find_various(db_session: &CachingSession) -> Result<(), CharybdisError> {
///         let date = Date::default();
///         let category_id = Uuid::new_v4();
///         let title = Text::default();
///
///         let posts: CharybdisModelStream<Post> = Post::find_by_date(date).execute(db_session).await?;
///         let posts: CharybdisModelStream<Post> = Post::find_by_date_and_category_id(date, category_id).execute(db_session).await?;
///         let posts: Post = Post::find_by_date_and_category_id_and_title(date, category_id, title.clone()).execute(db_session).await?;
///
///         let post: Post = Post::find_first_by_date(date).execute(db_session).await?;
///         let post: Post = Post::find_first_by_date_and_category_id(date, category_id).execute(db_session).await?;
///
///         let post: Option<Post> = Post::maybe_find_first_by_date(date).execute(db_session).await?;
///         let post: Option<Post> = Post::maybe_find_first_by_date_and_category_id(date, category_id).execute(db_session).await?;
///         let post: Option<Post> = Post::maybe_find_first_by_date_and_category_id_and_title(date, category_id, title.clone()).execute(db_session).await?;
///
///         // find by local secondary index
///         let posts: CharybdisModelStream<Post> = Post::find_by_date_and_title(date, title.clone()).execute(db_session).await?;
///         let post: Post = Post::find_first_by_date_and_title(date, title.clone()).execute(db_session).await?;
///         let post: Option<Post> = Post::maybe_find_first_by_date_and_title(date, title.clone()).execute(db_session).await?;
///
///         // find by global secondary index
///         let posts: CharybdisModelStream<Post> = Post::find_by_category_id(category_id).execute(db_session).await?;
///         let post: Post = Post::find_first_by_category_id(category_id).execute(db_session).await?;
///         let post: Option<Post> = Post::maybe_find_first_by_category_id(category_id).execute(db_session).await?;
///
///        Ok(())
///     }
/// }
/// ```
pub(crate) fn find_by_primary_keys_functions(
    struct_name: &syn::Ident,
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let table_name = ch_args.table_name();
    let comma_sep_cols = fields.db_fields.comma_sep_cols();
    let partition_keys_len = fields.partition_key_fields.len();
    let primary_key_stack = &fields.primary_key_fields;
    let mut generated = quote! {};

    for i in 0..primary_key_stack.len() {
        if i == MAX_FIND_BY_FIELDS {
            break;
        }

        let current_fields = primary_key_stack.iter().take(i + 1).cloned().collect::<Vec<&Field>>();

        // we need complete partition key to query
        if current_fields.len() < partition_keys_len {
            continue;
        }

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
        current_fields.push(lsi);

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

pub(crate) fn find_by_global_secondary_index(
    struct_name: &syn::Ident,
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let table_name = ch_args.table_name();
    let comma_sep_cols = fields.db_fields.comma_sep_cols();
    let gsi_fields = &fields.global_secondary_index_fields;

    let mut generated = quote! {};

    gsi_fields.iter().for_each(|gsi| {
        let query_str = format!("SELECT {} FROM {} WHERE {} = ?", comma_sep_cols, table_name, gsi.name);
        let find_fn = gsi.find_fn(struct_name, &query_str);
        let find_first_fn = gsi.find_first_fn(struct_name, &query_str);
        let maybe_find_first_fn = gsi.maybe_find_first_fn(struct_name, &query_str);

        generated.extend(find_fn);
        generated.extend(find_first_fn);
        generated.extend(maybe_find_first_fn);
    });

    generated
}
