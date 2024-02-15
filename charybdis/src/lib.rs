#![allow(async_fn_in_trait)]

pub mod batch;
pub mod callbacks;
pub mod errors;
pub mod iterator;
pub mod model;
pub mod operations;
pub mod query;
pub mod serializers;
pub mod stream;
pub mod types;

// orm macros
pub mod macros {
    pub use charybdis_macros::{
        char_model_field_attrs_gen, charybdis_model, charybdis_udt_model, charybdis_view_model,
    };
}

// scylla
pub use scylla::{
    cql_to_rust::{FromCqlVal, FromRow, FromRowError},
    frame::response::result::{CqlValue, Row},
    frame::value::ValueList,
    query::Query,
    serialize::row::SerializeRow,
    serialize::value::SerializeCql,
    statement::Consistency,
    transport::{errors::QueryError, session::TypedRowIter},
    CachingSession, QueryResult, Session,
};

// scylla macros
pub use scylla::macros::{FromRow, FromUserType, IntoUserType, SerializeCql, SerializeRow};
