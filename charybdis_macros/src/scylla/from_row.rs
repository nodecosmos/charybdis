use charybdis_parser::fields::CharybdisFields;
use quote::{quote, quote_spanned};
use syn::ImplItem;

pub(crate) fn from_row(struct_name: &syn::Ident, fields: &CharybdisFields) -> ImplItem {
    let fields_count: usize = fields.db_fields.len();

    let set_db_fields = fields.db_fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_type = &field.ty;

        quote_spanned! {field.span =>
            #field_ident: {
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

    let other_fields = fields.non_db_fields();
    let set_other_fields = other_fields.iter().map(|field| {
        let field_ident = &field.ident;

        quote_spanned! {field.span =>
            #field_ident: Default::default(),
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
