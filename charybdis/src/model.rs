use scylla::deserialize::row::DeserializeRow;
use scylla::serialize::row::SerializeRow;

pub trait BaseModel: Sized + SerializeRow + for<'frame, 'metadata> DeserializeRow<'frame, 'metadata> {
    // usually tuple of primary key values
    type PrimaryKey: SerializeRow + Send + Sync;
    type PartitionKey: SerializeRow + Send + Sync;

    const DB_MODEL_NAME: &'static str;
    const FIND_ALL_QUERY: &'static str;
    const FIND_BY_PRIMARY_KEY_QUERY: &'static str;
    const FIND_BY_PARTITION_KEY_QUERY: &'static str;
    const FIND_FIRST_BY_PARTITION_KEY_QUERY: &'static str;

    fn primary_key_values(&self) -> Self::PrimaryKey;
    fn partition_key_values(&self) -> Self::PartitionKey;
}

///
/// Model is a trait that defines the basic structure of a table in the database.
/// It is used to generate the necessary code for the charybdis orm.
/// Macro 'charybdis_model` generates the necessary code for implementation so you don't have
/// to write it manually. The macro is used in the following way:
/// ```rust
/// use charybdis::macros::charybdis_model;
/// use charybdis::types::{Text, Timestamp, Uuid};
///
/// #[charybdis_model(
///     table_name = users,
///     partition_keys = [id],
///     clustering_keys = [],
///     global_secondary_indexes = []
/// )]
/// pub struct User {
///     pub id: Uuid,
///     pub username: Text,
///     pub password: Text,
///     pub hashed_password: Text,
///     pub email: Text,
///     pub first_name: Option<Text>,
///     pub last_name: Option<Text>,
///     pub created_at: Timestamp,
///     pub updated_at: Timestamp,
/// }
///
/// ```
/// These structure is used by smart `migration` tool that automatically migrate the database
/// schema from the code.
/// It detects changes in the model and automatically applies the changes to the database.
///
/// If you have migration package installed, you can run the `migrate` command to automatically
/// migrate the database schema without having to write any CQL queries.
///
pub trait Model: BaseModel {
    const INSERT_QUERY: &'static str;
    const INSERT_IF_NOT_EXIST_QUERY: &'static str;
    const UPDATE_QUERY: &'static str;
    const DELETE_QUERY: &'static str;
    const DELETE_BY_PARTITION_KEY_QUERY: &'static str;
}

///
/// MaterializedView is a trait that defines the basic structure of materialized view.
/// It is used to generate the necessary code for the charybdis orm.
/// Macro 'charybdis_view_model` generates the necessary code for implementation
/// so you don't have to write it manually.
/// ```rust
/// use charybdis::macros::charybdis_view_model;
/// use charybdis::types::{Text, Timestamp, Uuid};
///
/// #[charybdis_view_model(
///     table_name=users_by_username,
///     base_table=users,
///     partition_keys=[username],
///     clustering_keys=[id]
/// )]
/// pub struct UsersByUsername {
///     pub username: Text,
///     pub id: Uuid,
///     pub email: Text,
///     pub created_at: Option<Timestamp>,
///     pub updated_at: Option<Timestamp>,
/// }
/// ```
/// Resulting auto-generated migration query will be:
///
/// ```sql
///  CREATE MATERIALIZED VIEW IF NOT EXISTS users_by_email
///  AS SELECT created_at, updated_at, username, email, id
///  FROM users
///  WHERE email IS NOT NULL AND id IS NOT NULL
///  PRIMARY KEY (email, id)
/// ```
///
pub trait MaterializedView: BaseModel {}

/// Declare udt model as a struct within `src/models/udts` dir:
/// ```rust
/// use charybdis::macros::charybdis_udt_model;
/// use charybdis::types::Text;
///
/// #[charybdis_udt_model(type_name = address)]
/// pub struct Address {
///     pub street: Text,
///     pub city: Text,
///     pub state: Text,
///     pub zip: Text,
///     pub country: Text,
/// }
/// ```
pub trait Udt: Sized + for<'frame, 'metadata> scylla::deserialize::value::DeserializeValue<'frame, 'metadata> {
    const DB_MODEL_NAME: &'static str;
}

pub trait TableOptions {
    const TABLE_OPTIONS: &'static str;
}

/// In extension of partial_model!() in case you need to build native model you can use `as_native` method:
/// ```rust
/// use charybdis::macros::charybdis_model;
/// use charybdis::types::{Text, Timestamp, Uuid};
/// use charybdis::model::AsNative;
///
/// #[charybdis_model(
///     table_name = users,
///     partition_keys = [id],
///     clustering_keys = [],
///     global_secondary_indexes = []
/// )]
/// #[derive(Default)]
/// pub struct User {
///     pub id: Uuid,
///     pub username: Text,
///     pub password: Text,
///     pub hashed_password: Text,
///     pub email: Text,
///     pub first_name: Option<Text>,
///     pub last_name: Option<Text>,
///     pub created_at: Timestamp,
///     pub updated_at: Timestamp,
/// }
///
/// partial_user!(UpdateUsernameUser, id, username);
///
/// impl UpdateUsernameUser {
///     pub fn native(&self) -> User {
///         self.as_native()
///     }
/// }
/// ```
pub trait AsNative<T: BaseModel> {
    fn as_native(&self) -> T;
}
