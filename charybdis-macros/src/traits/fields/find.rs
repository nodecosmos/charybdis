use crate::traits::fields::{FieldsNames, FieldsToArguments};
use crate::traits::string::ToIdent;
use crate::traits::tuple::Tuple;
use charybdis_parser::fields::Field;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) trait FieldsFindFnNames {
    fn find_by_fn_name(&self) -> String;
    fn find_first_by_fn_name(&self) -> String;
    fn maybe_find_first_by_fn_name(&self) -> String;
}

impl FieldsFindFnNames for Field {
    fn find_by_fn_name(&self) -> String {
        format!("find_by_{}", self.name)
    }

    fn find_first_by_fn_name(&self) -> String {
        format!("find_first_by_{}", self.name)
    }

    fn maybe_find_first_by_fn_name(&self) -> String {
        format!("maybe_find_first_by_{}", self.name)
    }
}

impl FieldsFindFnNames for Vec<Field> {
    fn find_by_fn_name(&self) -> String {
        format!("find_by_{}", self.names().join("_and_"))
    }

    fn find_first_by_fn_name(&self) -> String {
        format!("find_first_by_{}", self.names().join("_and_"))
    }

    fn maybe_find_first_by_fn_name(&self) -> String {
        format!("maybe_find_first_by_{}", self.names().join("_and_"))
    }
}

pub(crate) trait FieldsFindFn: FieldsFindFnNames + FieldsToArguments {
    fn find_fn(&self, struct_name: &syn::Ident, query_str: &String) -> TokenStream {
        let find_by_fn_name = self.find_by_fn_name().to_ident();
        let arguments = self.to_fn_args();
        let types_tp = arguments.types_tp();
        let values_tp = arguments.values_tp();

        quote! {
            pub fn #find_by_fn_name<'a>(
                #(#arguments),*
            ) -> charybdis::query::CharybdisQuery<'a, #types_tp, Self, charybdis::query::ModelStream<Self>> {
                <#struct_name as charybdis::operations::Find>::find(#query_str, #values_tp)
            }
        }
    }

    /// Generates a function that finds the first model that matches the query. Difference from
    /// `find_first` is that this function does not limit the number of results to 1 as it is only
    /// used when provided keys matches complete primary key.
    fn find_one_fn(&self, struct_name: &syn::Ident, query_str: &String) -> TokenStream {
        let find_by_fn_name = self.find_by_fn_name().to_ident();
        let arguments = self.to_fn_args();
        let types_tp = arguments.types_tp();
        let values_tp = arguments.values_tp();

        quote! {
            pub fn #find_by_fn_name<'a>(
                #(#arguments),*
            ) -> charybdis::query::CharybdisQuery<'a, #types_tp, Self, charybdis::query::ModelRow<Self>> {
                <#struct_name as charybdis::operations::Find>::find_first(#query_str, #values_tp)
            }
        }
    }
}

impl FieldsFindFn for Field {}
impl FieldsFindFn for Vec<Field> {}

pub(crate) trait FieldsFindFirstFns: FieldsFindFnNames + FieldsToArguments {
    fn find_first_fn(&self, struct_name: &syn::Ident, query_str: &String) -> TokenStream {
        let find_first_by_fn_name = self.find_first_by_fn_name().to_ident();
        let arguments = self.to_fn_args();
        let types_tp = arguments.types_tp();
        let values_tp = arguments.values_tp();
        let query_str = format!("{} LIMIT 1", query_str);

        quote! {
            pub fn #find_first_by_fn_name<'a>(
                #(#arguments),*
            ) -> charybdis::query::CharybdisQuery<'a, #types_tp, Self, charybdis::query::ModelRow<Self>> {
                <#struct_name as charybdis::operations::Find>::find_first(#query_str, #values_tp)
            }
        }
    }

    fn maybe_find_first_fn(&self, struct_name: &syn::Ident, query_str: &String) -> TokenStream {
        let maybe_find_first_by_fn_name = self.maybe_find_first_by_fn_name().to_ident();
        let arguments = self.to_fn_args();
        let types_tp = arguments.types_tp();
        let values_tp = arguments.values_tp();
        let query_str = format!("{} LIMIT 1", query_str);

        quote! {
            pub fn #maybe_find_first_by_fn_name<'a>(
                #(#arguments),*
            ) -> charybdis::query::CharybdisQuery<'a, #types_tp, Self, charybdis::query::OptionalModelRow<Self>> {
                <#struct_name as charybdis::operations::Find>::maybe_find_first(#query_str, #values_tp)
            }
        }
    }
}

impl FieldsFindFirstFns for Field {}
impl FieldsFindFirstFns for Vec<Field> {}
