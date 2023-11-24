use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Field, ImplItem};

pub(crate) fn from_row(struct_name: &syn::Ident, db_fields: &Vec<Field>, all_fields: &Vec<Field>) -> ImplItem {
    let fields_count: usize = db_fields.len();

    let set_db_fields = db_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        quote_spanned! {field.span() =>
            #field_name: {
                let (col_ix, col_value) = vals_iter
                    .next()
                    .unwrap(); // vals_iter size is checked before this code is reached, so
                               // it is safe to unwrap

                <#field_type as FromCqlVal<::std::option::Option<CqlValue>>>::from_cql(col_value)
                    .map_err(|e| FromRowError::BadCqlVal {
                        err: e,
                        column: col_ix,
                    })?
            },
        }
    });

    let set_other_fields = all_fields
        .iter()
        .filter(|field| !db_fields.contains(field))
        .map(|field| {
            let field_name = &field.ident;

            quote_spanned! {field.span() =>
                #field_name: Default::default(),
            }
        });

    let generated = quote! {
        fn from_row(row: charybdis::Row) -> ::std::result::Result<Self, charybdis::FromRowError> {
                use charybdis::{CqlValue, FromCqlVal, FromRow, FromRowError};
                use ::std::result::Result::{Ok, Err};
                use ::std::iter::{Iterator, IntoIterator};

                if #fields_count != row.columns.len() {
                    return Err(FromRowError::WrongRowSize {
                        expected: #fields_count,
                        actual: row.columns.len(),
                    });
                }
                let mut vals_iter = row.columns.into_iter().enumerate();

                Ok(#struct_name {
                    #(#set_db_fields)*
                    #(#set_other_fields)*
                })
            }
    };

    syn::parse_quote!(#generated)
}
