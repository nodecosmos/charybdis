use proc_macro2::TokenStream;
use quote::quote;

use charybdis_parser::fields::{CharybdisFields, Field};
use charybdis_parser::traits::CharybdisMacroArgs;

use crate::traits::fields::{FieldsNames, FieldsQuery, FieldsToArguments};
use crate::traits::tuple::Tuple;

const MAX_DELETE_BY_FUNCTIONS: usize = 3;

/// for 3 keys generate additional function that deletes by partition key & partial clustering key
/// Example:
/// ```rust
/// use charybdis::macros::charybdis_model;
/// use charybdis::errors::CharybdisError;
/// use charybdis::types::{Timestamp, Uuid};
/// use scylla::CachingSession;
///
/// #[charybdis_model(
///     table_name = users,
///     partition_keys = [id],
///     clustering_keys = [org_id, created_at],
///     global_secondary_indexes = []
/// )]
/// pub struct User {
///     id: Uuid,
///     org_id: Uuid,
///     created_at: Timestamp,
/// }
/// impl User {
///     pub async fn delete_funs(db: &CachingSession) -> Result<(), CharybdisError>  {
///         let id = Uuid::new_v4();
///         let org_id = Uuid::new_v4();
///         let created_at = chrono::Utc::now();
///
///         User::delete_by_id(id).execute(db).await?;
///         User::delete_by_id_and_org_id(id, org_id).execute(db).await?;
///         User::delete_by_id_and_org_id_and_created_at(id, org_id, created_at).execute(db).await?;
///
///         Ok(())
///     }
/// }
/// ```
pub(crate) fn delete_by_primary_key_functions(ch_args: &CharybdisMacroArgs, fields: &CharybdisFields) -> TokenStream {
    let table_name = ch_args.table_name();
    let partition_keys_len = fields.partition_key_fields.len();
    let primary_key_stack = &fields.primary_key_fields;
    let mut generated = quote! {};

    for i in 0..primary_key_stack.len() {
        if i == MAX_DELETE_BY_FUNCTIONS {
            break;
        }

        let current_fields = primary_key_stack.iter().take(i + 1).cloned().collect::<Vec<&Field>>();

        // we need complete partition key to query
        if current_fields.len() < partition_keys_len {
            continue;
        }

        let query_str = format!(
            "DELETE FROM {} WHERE {}",
            table_name,
            current_fields.where_placeholders()
        );
        let find_by_fun_name_str = format!("delete_by_{}", current_fields.names().join("_and_"));
        let delete_by_fun_name = syn::Ident::new(&find_by_fun_name_str, proc_macro2::Span::call_site());
        let arguments = current_fields.to_fn_args();
        let types_tp = arguments.types_tp();
        let values_tp = arguments.values_tp();

        let generated_func = quote! {
            pub fn #delete_by_fun_name<'a>(
                #(#arguments),*
            ) -> charybdis::query::CharybdisQuery<'a, #types_tp, Self, charybdis::query::ModelMutation> {
                charybdis::query::CharybdisQuery::new(#query_str, charybdis::query::QueryValue::Owned(#values_tp))
            }
        };

        generated.extend(generated_func);
    }

    generated
}
