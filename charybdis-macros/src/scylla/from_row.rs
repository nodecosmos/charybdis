use quote::{quote, quote_spanned};
use syn::ImplItem;

use charybdis_parser::fields::CharybdisFields;

/// Generates the `from_row` method for the struct. We have manually implemented this method
/// because we need to enable non-db fields to be set to their default values.
pub(crate) fn from_row(struct_name: &syn::Ident, fields: &CharybdisFields) -> ImplItem {
    let fields_count: usize = fields.db_fields.len();

    let set_db_fields = fields.db_fields.iter().enumerate().map(|(col_ix, field)| {
        let field_ident = &field.ident;
        let field_type = &field.ty;

        quote_spanned! {field.span =>
            #field_ident: {
                let col_value = ::std::mem::take(&mut row_columns[#col_ix]);
                <#field_type as charybdis::scylla::FromCqlVal<::std::option::Option<charybdis::scylla::CqlValue>>>::from_cql(col_value)
                    .map_err(|e| charybdis::scylla::FromRowError::BadCqlVal {
                        err: e,
                        column: #col_ix,
                    })?
            },
        }
    });

    let other_fields = fields.non_db_fields();
    let set_other_fields = other_fields.iter().map(|field| {
        let field_ident = &field.ident;

        quote_spanned! {field.span =>
            #field_ident: Default::default(),
        }
    });

    let generated = quote! {
        fn from_row(row: charybdis::scylla::Row) -> ::std::result::Result<Self, charybdis::scylla::FromRowError> {
            use ::std::result::Result::{Ok, Err};
            use ::std::iter::{Iterator, IntoIterator};

            let row_columns_len = row.columns.len();

            let mut row_columns: [_; #fields_count] = row.columns.try_into().map_err(|_|
                charybdis::scylla::FromRowError::WrongRowSize {
                    expected: #fields_count,
                    actual: row_columns_len,
                }
            )?;

            Ok(#struct_name {
                #(#set_db_fields)*
                #(#set_other_fields)*
            })
        }
    };

    syn::parse_quote!(#generated)
}
