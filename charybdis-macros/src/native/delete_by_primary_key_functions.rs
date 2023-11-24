use crate::utils::{serialized_value_adder, struct_fields_to_fn_args};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;

const MAX_DELETE_BY_FUNCTIONS: usize = 3;

/// for 3 keys generate additional function that deletes by partition key & partial clustering key
/// Example:
/// ```ignore
/// #[charybdis_model(
///     table_name = users,
///     partition_keys = [id],
///     clustering_keys = [org_id, created_at],
///     global_secondary_indexes = [])]
/// pub struct UserOps {...}
/// ```
/// we would have a functions:
/// ```ignore
///  User::delete_by_id_and_org_id(session: &Session, org_id: Uuid) -> Result<Vec<User>, errors::CharybdisError>
///  User::delete_by_id_and_org_id_and_created_at(session: &Session, org_id: Uuid, created_at: Timestamp) -> Result<Vec<User>, errors::CharybdisError>
pub(crate) fn delete_by_primary_key_functions(
    ch_args: &CharybdisMacroArgs,
    fields: &Vec<Field>,
    struct_name: &syn::Ident,
) -> TokenStream {
    let table_name = ch_args.table_name.clone().unwrap();

    let mut primary_key_stack = ch_args.primary_key();
    let mut generated = quote! {};

    let mut i = 0;

    while !primary_key_stack.is_empty() {
        if i > MAX_DELETE_BY_FUNCTIONS {
            break;
        }

        i += 1;

        let current_keys = primary_key_stack.clone();
        let primary_key_where_clause: String = current_keys.join(" = ? AND ");
        let query_str = format!("DELETE FROM {} WHERE {} = ?", table_name, primary_key_where_clause);
        let find_by_fun_name_str = format!(
            "delete_by_{}",
            current_keys
                .iter()
                .map(|key| key.to_string())
                .collect::<Vec<String>>()
                .join("_and_")
        );
        let delete_by_fun_name = syn::Ident::new(&find_by_fun_name_str, proc_macro2::Span::call_site());
        let arguments = struct_fields_to_fn_args(struct_name.to_string(), fields.clone(), current_keys.clone());
        let capacity = current_keys.len();
        let serialized_adder = serialized_value_adder(current_keys.clone());

        let generated_func = quote! {
            pub async fn #delete_by_fun_name(
                session: &charybdis::CachingSession,
                #(#arguments),*
            ) -> Result<charybdis::QueryResult, charybdis::errors::CharybdisError> {
                use futures::TryStreamExt;

                let mut serialized = charybdis::SerializedValues::with_capacity(#capacity);

                #serialized_adder

                let query_result = session.execute(#query_str, serialized).await?;

                Ok(query_result)
            }
        };

        primary_key_stack.pop();

        generated.extend(generated_func);
    }

    generated
}
