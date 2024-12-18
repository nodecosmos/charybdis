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

#[cfg(feature = "migrate")]
pub use migrate;

pub mod macros {
    pub use charybdis_macros::{
        char_model_field_attrs_gen, charybdis_model, charybdis_udt_model, charybdis_view_model,
    };

    pub mod scylla {
        pub use scylla::macros::{
            DeserializeRow, DeserializeValue, FromRow, FromUserType, IntoUserType, SerializeRow, SerializeValue,
        };
    }
}

pub mod scylla {
    pub use scylla::frame::response::result::{CqlValue, Row};
    pub use scylla::serialize::value::SerializeValue;
    pub use scylla::statement::PagingState;
    pub use scylla::transport::iterator::TypedRowStream;
    pub use scylla::*;
}

pub mod options {
    pub use scylla::execution_profile::ExecutionProfileHandle;
    pub use scylla::history::HistoryListener;
    pub use scylla::retry_policy::RetryPolicy;
    pub use scylla::statement::{Consistency, SerialConsistency};
}
