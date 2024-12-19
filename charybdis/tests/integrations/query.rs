use crate::common::db_session;
use crate::models::{Address, User};
use charybdis::operations::Insert;

#[tokio::test]
async fn create() {
    let user = User {
        id: uuid::Uuid::new_v4(),
        username: "test".to_string(),
        email: "homer@simpson.com".to_string(),
        password: "Marge".to_string(),
        first_name: "Homer".to_string(),
        last_name: "Simpson".to_string(),
        bio: Some("I like donuts".to_string()),
        address: Some(Address {
            street: "742 Evergreen Terrace".to_string(),
            city: "Springfield".to_string(),
            state: "Illinois".to_string(),
            zip: "62701".to_string(),
            country: "USA".to_string(),
        }),
        is_confirmed: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let db_session = db_session().await;

    user.insert().execute(db_session).await.expect("Failed to insert user");
}
