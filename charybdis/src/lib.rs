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

pub mod macros {
    pub use charybdis_macros::{
        char_model_field_attrs_gen, charybdis_model, charybdis_udt_model, charybdis_view_model,
    };

    pub mod scylla {
        pub use scylla::macros::{FromRow, FromUserType, IntoUserType, SerializeRow, SerializeValue};
    }
}

pub mod scylla {
    pub use scylla::frame::response::cql_to_rust::{FromCqlVal, FromRow, FromRowError};
    pub use scylla::frame::response::result::{CqlValue, Row};
    pub use scylla::serialize::value::SerializeValue;
    pub use scylla::statement::PagingState;
}

pub mod options {
    pub use scylla::execution_profile::ExecutionProfileHandle;
    pub use scylla::history::HistoryListener;
    pub use scylla::retry_policy::RetryPolicy;
    pub use scylla::statement::{Consistency, SerialConsistency};
}
