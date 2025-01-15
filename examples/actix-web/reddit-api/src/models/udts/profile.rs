use charybdis::macros::charybdis_udt_model;
use charybdis::types::Text;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
#[charybdis_udt_model(type_name = profile)]
pub struct Profile {
    pub first_name: Text,
    pub last_name: Text,
    pub username: Text,
    pub email: Text,
}

#[cfg(test)]
impl Profile {
    pub fn sample() -> Self {
        Self {
            first_name: "Homer".to_string(),
            last_name: "Simpson".to_string(),
            username: "homer".to_string(),
            email: "homer@simpson.com".to_string(),
        }
    }
}
