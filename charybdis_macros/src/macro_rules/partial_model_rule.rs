use crate::utils::camel_to_snake_case;
use charybdis_parser::fields::CharybdisFields;
use charybdis_parser::macro_args::CharybdisMacroArgs;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{parse_str, Attribute, DeriveInput, Field};

///
/// ## Generates macro rule for partial model generation
///
/// E.g.: if we have a model User with some fields, it will append partial_<model_name>! macro that
/// can be used to generate a partial model struct with only some fields that can be used for ORM.
/// It is basically the same as base model struct, but with provided fields only.
///
/// So, if we have a model User with some fields:
/// ```ignore
/// use charybdis::*;
/// use super::Address;
/// #[charybdis_model(
///     table_name = "users",
///     partition_keys = ["id"],
///     clustering_keys = []
///     global_secondary_indexes = [])]
/// pub struct User {
///     pub id: Uuid,
///     pub username: Text,
///     pub password: Text,
///     pub hashed_password: Text,
///     pub email: Text,
///     pub created_at: Timestamp,
///     pub updated_at: Timestamp,
///     pub address: Address,
/// }
/// ```
/// It will generate a `partial_user!` macro that can be used to generate a partial users structs.
///
/// ## Example generation:
/// ```ignore
/// partial_user!(PartialUser, id, username, email);
/// ```
/// It will generate a struct with only those fields:
/// ```ignore
/// #[charybdis_model(
///     table_name = "users",
///     partition_keys = ["id"],
///     clustering_keys = []
///     global_secondary_indexes = [])]
/// pub struct PartialUser {
///    pub id: Uuid,
///    pub username: Text,
///    pub email: Text,
/// }
///```
/// And we can use it as a normal model struct.
///
/// ```ignore
/// let mut partial_user = PartialUser {id, username, email};
/// partial_user.insert(&session).await?;
/// partial_user.find_by_primary_key(&session).await?;
/// ```
///
/// ## Example usage:
/// ```ignore
///     partial_user!(PartialUser, id, username, email);
///
///     let mut partial_user = PartialUser {
///         id: Uuid::new_v4(),
///         username: "test".to_string(),
///         email: "test@gmail.com".to_string(),
///     };
///
///    println!("{:?}", partial_user);
///```
///---
///
/// ### `#[charybdis_model]` declaration
/// It also appends `#[charybdis_model(...)]` declaration with clustering keys and secondary indexes
/// based on fields that are provided in partial_model struct.
///
/// E.g. if we have model:
/// ```ignore
/// #[partial_model_generator]
/// #[charybdis_model(
///     table_name = users,
///     partition_keys = [id],
///     clustering_keys = [created_at, updated_at],
///     global_secondary_indexes = []
/// )]
/// pub struct User {
///     pub id: Uuid,
///     pub username: Text,
///     pub password: Text,
///     pub hashed_password: Text,
///     pub email: Text,
///     pub created_at: Timestamp,
///     pub updated_at: Timestamp,
/// }
/// ```
///
/// and we use partial model macro:
/// ```ignore
/// partial_user!(UserOps, id, username, email, created_at);
/// ```
/// it will generate a struct with `#[charybdis_model(...)]` declaration:
///
/// ```ignore
/// #[charybdis_model(
///     table_name = users,
///     partition_keys = [id],
///     clustering_keys = [created_at],
///     global_secondary_indexes = [])]
/// pub struct UserOps {...}
/// ```
/// Note that `updated_at` is not present in generated declaration.
/// However, all partition keys are required for db operations, so we can't have partial partition
/// keys.
///

pub(crate) fn partial_model_macro_generator(args: CharybdisMacroArgs, input: &DeriveInput) -> TokenStream {
    let fields = CharybdisFields::from_input(&input).all_fields();

    if args.exclude_partial_model.unwrap_or(false) {
        return TokenStream::new();
    }

    // TODO: For models without all clustering keys, this will panic!
    //       we should add support for partial clustering keys.

    // TODO: Enable better error handling for this macro as it hard to debug
    //       nested macros when they panic.

    // macro names (avoiding name collisions)
    let native_struct = &input.ident;
    let struct_name_str = camel_to_snake_case(&native_struct.to_string());

    let macro_name_str = format!("partial_{}", struct_name_str);
    let macro_name = parse_str::<TokenStream>(&macro_name_str).unwrap();

    let field_types_hash = build_field_types_hash(&fields);
    let field_attributes_hash = build_field_attributes_hash(&fields);

    let table_name = args.table_name.unwrap().to_token_stream();

    let cks = args.clustering_keys.unwrap_or(vec![]);
    let pks = args.partition_keys.unwrap_or(vec![]);
    let sec_idxes = args.global_secondary_indexes.unwrap_or(vec![]);

    let cks: Vec<syn::Ident> = cks
        .into_iter()
        .map(|s| syn::Ident::new(&s, proc_macro2::Span::call_site()))
        .collect();

    let pks: Vec<syn::Ident> = pks
        .into_iter()
        .map(|s| syn::Ident::new(&s, proc_macro2::Span::call_site()))
        .collect();

    let sec_idxes: Vec<syn::Ident> = sec_idxes
        .into_iter()
        .map(|s| syn::Ident::new(&s, proc_macro2::Span::call_site()))
        .collect();

    // attributes that are not charybdis_model
    let other_attrs = &input
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("charybdis_model"))
        .collect::<Vec<&Attribute>>();

    let expanded: TokenStream = quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($struct_name:ident, $($field:ident),*) => {
                #[charybdis::macros::char_model_field_attrs_gen(
                    fields_names=[$($field),*],
                    field_types_hash=#field_types_hash,
                    field_attributes_hash=#field_attributes_hash
                )]
                #[charybdis::macros::charybdis_model(
                    table_name=#table_name,
                    partition_keys=[ #(#pks),* ],
                    clustering_keys=[ #(#cks),* ],
                    global_secondary_indexes=[ #(#sec_idxes),* ],
                    exclude_partial_model=true
                )]
                #(#other_attrs)*
                pub struct $struct_name {}

                impl charybdis::model::AsNative<#native_struct> for $struct_name {
                    fn as_native(&self) -> #native_struct {
                        use charybdis::operations::New;

                        let mut new_model = #native_struct::new();

                        $(
                            new_model.$field = self.$field.clone();
                        )*

                        new_model
                    }
                }
            };
        }
        pub(crate) use #macro_name;
    };

    expanded
}

/// field_types_hash -> key is field name and value is field attributes.
fn build_field_types_hash(fields: &Vec<Field>) -> String {
    let mut field_types = quote! {};

    for field in fields.iter() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        field_types.extend(quote! { #name => #ty; });
    }

    field_types.to_string().replace('\n', "")
}

/// field_attributes_hash -> key is field name and value is field attributes.
fn build_field_attributes_hash(fields: &Vec<Field>) -> String {
    let mut field_attributes = quote! {};

    for field in fields.iter() {
        let name = field.ident.as_ref().unwrap();
        let attrs: &Vec<Attribute> = &field.attrs;

        field_attributes.extend(quote! { #name => #(#attrs)*; });
    }

    // strip newlines
    field_attributes.to_string().replace('\n', "")
}

/// Used to append subset of fields to model struct.
/// Arguments `field_attributes_hash` and `field_types_hash` are generated by
/// `charybdis_macros::partial_model_macro_generator` macro.
/// field_attributes_hash -> key is field name and value is field attributes.
/// field_types_hash -> key is field name and value is field type and generates a struct with
pub fn char_model_field_attrs_macro_gen(args: CharybdisMacroArgs, input: DeriveInput) -> TokenStream {
    let input_attributes = &input.attrs;

    let struct_name = &input.ident;
    let field_names = args
        .fields_names
        .unwrap_or_else(|| panic!("failed to parse field names: {}", struct_name));

    let field_types_hash = args
        .field_types_hash
        .unwrap_or_else(|| panic!("failed to parse field types hash for struct: {}", struct_name));

    let field_attributes_hash = args.field_attributes_hash.unwrap_or(HashMap::new());

    let fields_tokens = field_names
        .iter()
        .map(|field_name| {
            let field_name_token: TokenStream = parse_str(field_name).unwrap();
            let field_type = field_types_hash.get(field_name).unwrap_or_else(|| {
                panic!(
                    "failed to parse field type for field: {} in struct: {}",
                    field_name, struct_name
                )
            });

            let empty = parse_str("").unwrap();
            let field_attributes = field_attributes_hash.get(field_name).unwrap_or(&empty);

            quote! {
                #field_attributes
                pub #field_name_token: #field_type
            }
        })
        .collect::<Vec<TokenStream>>();

    let expanded = quote! {
        #(#input_attributes)*
        pub struct #struct_name {
            #(#fields_tokens),*
        }
    };

    expanded
}
