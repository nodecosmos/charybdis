use crate::utils::{struct_fields_to_fn_args, where_placeholders, Tuple};
use charybdis_parser::fields::{CharybdisFields, Field};
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::quote;

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
    struct_name: &syn::Ident,
    ch_args: &CharybdisMacroArgs,
    fields: &CharybdisFields,
) -> TokenStream {
    let table_name = ch_args.table_name();

    let primary_key_stack = &fields.primary_key_fields;
    let mut generated = quote! {};

    for i in 0..primary_key_stack.len() {
        if i == MAX_DELETE_BY_FUNCTIONS {
            break;
        }

        let current_fields = primary_key_stack.iter().take(i + 1).cloned().collect::<Vec<Field>>();
        let current_field_names = current_fields
            .iter()
            .map(|field| field.name.clone())
            .collect::<Vec<String>>();
        let query_str = format!(
            "DELETE FROM {} WHERE {}",
            table_name,
            where_placeholders(&current_fields)
        );
        let find_by_fun_name_str = format!("delete_by_{}", current_field_names.join("_and_"));
        let delete_by_fun_name = syn::Ident::new(&find_by_fun_name_str, proc_macro2::Span::call_site());
        let arguments = struct_fields_to_fn_args(
            struct_name.to_string(),
            fields.db_fields.clone(),
            current_field_names.clone(),
        );
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
