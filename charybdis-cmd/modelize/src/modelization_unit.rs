use charybdis_parser::schema::SchemaObject;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum ModelizationObjectType {
    Udt,
    Table,
    MaterializedView,
}

impl Display for ModelizationObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelizationObjectType::Udt => write!(f, "UDT"),
            ModelizationObjectType::Table => write!(f, "Table"),
            ModelizationObjectType::MaterializedView => write!(f, "Materialized View"),
        }
    }
}

pub(crate) struct ModelizationUnit<'a> {
    model_name: &'a String,
    object_type: ModelizationObjectType,
    db_schema: &'a SchemaObject,
}

/// create directory models/udts if it doesn't exist
/// create file models/udts/<udt_name>.rs if it doesn't exist
/// write the modelization code in the file
/// use following code as a template:
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
///
/// create directory models if it doesn't exist
/// create file models/<table_name>.rs if it doesn't exist
/// write the modelization code in the file
/// use following code as a template:
/// pub use super::udts::Address;
/// use charybdis::macros::charybdis_model;
/// use charybdis::types::{Boolean, Set, Text, Timestamp, Uuid};
///
/// #[charybdis_model(
///     table_name = users,
///     partition_keys = [id],
///     clustering_keys = [],
///     global_secondary_indexes = [username, email],
///     local_secondary_indexes = [
///         ([id], [email])
///     ]
/// )]
/// pub struct User {
///     #[serde(default = "Uuid::new_v4")]
///     pub id: Uuid,
///     pub username: Text,
///     pub email: Text,
///     pub password: Text,
///     pub first_name: Text,
///     pub last_name: Text,
///     pub bio: Option<Text>,
///     pub created_at: Option<Timestamp>,
///     pub updated_at: Option<Timestamp>,
///     pub address: Option<Address>,
///     pub is_confirmed: Boolean,
///     pub is_blocked: Boolean,
///     pub liked_object_ids: Option<Set<Uuid>>,
/// }
///
/// create directory models/materialized_views if it doesn't exist
/// create file models/materialized_views/<materialized_view_name>.rs if it doesn't exist
/// write the modelization code in the file
/// use following code as a template:
/// use charybdis::macros::charybdis_view_model;
/// use charybdis::types::{Text, Timestamp, Uuid};
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
///
impl<'a> ModelizationUnit<'a> {
    pub(crate) fn new(
        model_name: &'a String,
        object_type: ModelizationObjectType,
        db_schema: &'a SchemaObject,
    ) -> Self {
        Self {
            model_name,
            object_type,
            db_schema,
        }
    }

    pub(crate) async fn run(&self) {
        match self.object_type {
            ModelizationObjectType::Udt => {
                println!(
                    "Modelizing {} {}...",
                    self.object_type.to_string().to_lowercase(),
                    self.model_name
                );

                println!(
                    "Modelized {} {} successfully!",
                    self.object_type.to_string().to_lowercase(),
                    self.model_name
                );
            }
            ModelizationObjectType::Table => {}
            ModelizationObjectType::MaterializedView => {}
        }
    }
}
