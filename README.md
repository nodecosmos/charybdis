# Rust ORM for ScyllaDB and Apache Cassandra

### Use monstrous tandem of scylla and charybdis for your next project

⚠️ This project is currently in an early stage of development. Feedback and contributions are
welcomed!

[![Crates.io](https://img.shields.io/crates/v/charybdis)](https://crates.io/crates/charybdis)
[![Docs.rs](https://docs.rs/charybdis/badge.svg)](https://docs.rs/charybdis)
[![License](https://img.shields.io/crates/l/charybdis)]()

<p>
<img src="https://www.scylladb.com/wp-content/uploads/scylla-opensource-1.png" width="150" alt="scylla_logo">
<img src="https://upload.wikimedia.org/wikipedia/commons/5/5e/Cassandra_logo.svg" width="150" alt="cassandra_logo">
</p>

#### Charybdis is a ORM layer on top of `scylla_rust_driver` focused on easy of use and performance

## Breaking changes

As of `0.4.13` UDT fields must be in the same order as they are in the database. This is due to
scylla driver limitation that does not support named bind values. Earlier versions would
automatically order fields by name, but this is no longer the case as ORM could not work with
exiting UDTs.

## Usage considerations:

- Provide and expressive API for CRUD & Complex Query operations on model as a whole
- Provide easy way to work with subset of model fields by using automatically
  generated `partial_<model>!` macro
- Provide easy way to run complex queries by using automatically generated `find_<model>!` macro
- Automatic migration tool analyzes the project files and runs migrations according to differences
  between the model definition and database

## Performance consideration:

- It uses prepared statements (shard/token aware) -> bind values
- It expects `CachingSession` as a session arg for operations
- Queries are macro generated str constants (no concatenation at runtime)
- By using `find_<model>!` macro we can run complex queries that are generated at compile time
  as `&'static str`
- Although it has expressive API it's thin layer on top of scylla_rust_driver, and it does not
  introduce any significant
  overhead

## Table of Contents

- [Charybdis Models](#charybdis-models)
    - [Define Tables](#define-tables)
    - [Define UDTs](#Define-UDT)
    - [Define Materialized Views](#Define-Materialized-Views)
- [Automatic migration with `charybdis-migrate`](#automatic-migration)
- [Basic Operations](#basic-operations)
    - [Insert](#insert)
    - [Find](#find)
        - [Find by primary key](#find-by-primary-key)
        - [Find by partition key](#find-by-partition-key)
        - [Find by primary key associated](#find-by-primary-key-associated)
        - [Available find functions](#available-find-functions)
        - [Custom filtering](#custom-filtering)
    - [Update](#update)
    - [Delete](#delete)
        - [Macro generated delete helpers](#macro-generated-delete-helpers)
        - [Custom delete queries](#custom-delete-queries)
- [Configuration Options](#configuration)
- [Batch Operations](#batch-operations)
    - [Chunked Batch Operations](#chunked-batch-operations)
    - [Batch Configuration](#batch-configuration)
- [Partial Model](#partial-model)
    - [Considerations](#partial-model-considerations)
    - [As Native](#as-native)
- [Callbacks](#callbacks)
    - [Implementation](#implementation)
    - [Triggering Callbacks](#triggering-callbacks)
- [Collection](#collections)
    - [Generated Collection Queries](#generated-collection-queries)
    - [Generated Collection Methods](#generated-collection-methods)
- [Ignored fields](#ignored-fields)
- [Roadmap](#Roadmap)

## Charybdis Models

### Define Tables

  ```rust
  use charybdis::macros::charybdis_model;
  use charybdis::types::{Text, Timestamp, Uuid};
  
  #[charybdis_model(
      table_name = users,
      partition_keys = [id],
      clustering_keys = [],
      global_secondary_indexes = [username],
      local_secondary_indexes = [],
      static_columns = []
  )]
  pub struct User {
      pub id: Uuid,
      pub username: Text,
      pub email: Text,
      pub created_at: Timestamp,
      pub updated_at: Timestamp,
      pub address: Address,
  }
  ```

### Define UDT

 ```rust
  use charybdis::macros::charybdis_udt_model;
  use charybdis::types::Text;
  
  #[charybdis_udt_model(type_name = address)]
  pub struct Address {
      pub street: Text,
      pub city: Text,
      pub state: Option<Text>,
      pub zip: Text,
      pub country: Text,
  }
  ```

🚨 [UDT fields must be in the same order as they are in the database](https://rust-driver.docs.scylladb.com/stable/data-types/udt.html).

Note that in order for migration to correctly detect changes on each migration, `type_name` has to
match struct name. So if we have `struct ReorderData` we have to use
`#[charybdis_udt_model(type_name = reorderdata)]` - without underscores.

### Define Materialized Views

  ```rust
  use charybdis::macros::charybdis_view_model;
  use charybdis::types::{Text, Timestamp, Uuid};
  
  #[charybdis_view_model(
      table_name=users_by_username,
      base_table=users,
      partition_keys=[username],
      clustering_keys=[id]
  )]
  pub struct UsersByUsername {
      pub username: Text,
      pub id: Uuid,
      pub email: Text,
      pub created_at: Timestamp,
      pub updated_at: Timestamp,
  }
  
  ```

Resulting auto-generated migration query will be:

  ```sql
  CREATE MATERIALIZED VIEW IF NOT EXISTS users_by_email
  AS SELECT created_at, updated_at, username, email, id
  FROM users
  WHERE email IS NOT NULL AND id IS NOT NULL
  PRIMARY KEY (email, id)
  ```

## Automatic migration

* <a name="automatic-migration"></a>
  `charybdis-migrate` enables automatic migration to database without need to write migrations by
  hand.
  It iterates over project files and generates migrations based on differences between model
  definitions and database.
  It supports following operations:
    - Create new tables
    - Create new columns
    - Drop columns
    - Change field types (drop and recreate column `--drop-and-replace` flag)
    - Create secondary indexes
    - Drop secondary indexes
    - Create UDTs
    - Create materialized views
    - Table options
      ```rust
        #[charybdis_model(
            table_name = commits,
            partition_keys = [object_id],
            clustering_keys = [created_at, id],
            global_secondary_indexes = [],
            local_secondary_indexes = [],
            table_options = #r"
                WITH CLUSTERING ORDER BY (created_at DESC) 
                AND gc_grace_seconds = 86400
            ";
        )]
        #[derive(Serialize, Deserialize, Default)]
        pub struct Commit {...}
        ```
        * ⚠️ If table exists, table options will result in alter table query that without
          `CLUSTERING ORDER` and `COMPACT STORAGE` options.

  Model dropping is not added. If you removed model, you need to drop table manuall

* ### Running migration
  ```bash
  cargo install charybdis-migrate
  
  migrate --hosts <host> --keyspace <your_keyspace> --drop-and-replace (optional)
  ```

  ⚠️ If you are working with **existing** datasets, before running migration you need to make sure
  that your **model
  **
  definitions structure matches the database in respect to table names, column names, column types,
  partition keys,
  clustering keys and secondary indexes so you don't alter structure accidentally.
  If structure is matched, it will not run any migrations. As mentioned above,
  in case there is no model definition for table, it will **not** drop it. In future,
  we will add `modelize` command that will generate `src/models` files from existing data source.

* ### Global secondary indexes
  If we have model:
  ```rust
  #[charybdis_model(
      table_name = users,
      partition_keys = [id],
      clustering_keys = [],
      global_secondary_indexes = [username]
  )]
  ```
  resulting query will be: `CREATE INDEX ON users (username);`

* ### Local secondary Indexes

  Indexes that are scoped to the partition key
    ```rust
    #[charybdis_model(
        table_name = menus,
        partition_keys = [location],
        clustering_keys = [name, price, dish_type],
        global_secondary_indexes = [],
        local_secondary_indexes = [dish_type]
    )]
    ```
  resulting query will be: `CREATE INDEX ON menus((location), dish_type);`

## Basic Operations:

For each operation you need to bring respective trait into scope. They are defined
in `charybdis::operations` module.

### Insert

- ```rust
  use charybdis::{CachingSession, Insert};
  
  #[tokio::main]
  async fn main() {
    let session: &CachingSession; // init sylla session
    
    // init user
    let user: User = User {
      id,
      email: "charybdis@nodecosmos.com".to_string(),
      username: "charybdis".to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      address: Some(
          Address {
              street: "street".to_string(),
              state: "state".to_string(),
              zip: "zip".to_string(),
              country: "country".to_string(),
              city: "city".to_string(),
          }
      ),
    };
  
    // create
    user.insert().execute(&session).await;
  }
  ```

## Find

- ### Find by primary key
  ```rust
    let user = User {id, ..Default::default()};
    let user = user.find_by_primary_key().execute(&session).await?;
  ```
- ### Find by partition key

  ```rust
    let users =  User {id, ..Default::default()}.find_by_partition_key().execute(&session).await;
  ```
- ### Find by primary key associated
  ```rust
  let users = User::find_by_primary_key_value(val: User::PrimaryKey).execute(&session).await;
  ```
- ### Available find functions
  ```rust
  use scylla::CachingSession;
  use charybdis::errors::CharybdisError;
  use charybdis::macros::charybdis_model;
  use charybdis::stream::CharybdisModelStream;
  use charybdis::types::{Date, Text, Uuid};
  
  #[charybdis_model(
      table_name = posts,
      partition_keys = [date],
      clustering_keys = [category_id, title],
      global_secondary_indexes = [category_id],
      local_secondary_indexes = [title]
  )]
  pub struct Post {
      pub date: Date,
      pub category_id: Uuid,
      pub title: Text,
  }
  
  impl Post {
      async fn find_various(db_session: &CachingSession) -> Result<(), CharybdisError> {
         let date = Date::default();
         let category_id = Uuid::new_v4();
         let title = Text::default();
      
         let posts: CharybdisModelStream<Post> = Post::find_by_date(date).execute(db_session).await?;
         let posts: CharybdisModelStream<Post> = Post::find_by_date_and_category_id(date, category_id).execute(db_session).await?;
         let posts: Post = Post::find_by_date_and_category_id_and_title(date, category_id, title.clone()).execute(db_session).await?;
      
         let post: Post = Post::find_first_by_date(date).execute(db_session).await?;
         let post: Post = Post::find_first_by_date_and_category_id(date, category_id).execute(db_session).await?;
      
         let post: Option<Post> = Post::maybe_find_first_by_date(date).execute(db_session).await?;
         let post: Option<Post> = Post::maybe_find_first_by_date_and_category_id(date, category_id).execute(db_session).await?;
         let post: Option<Post> = Post::maybe_find_first_by_date_and_category_id_and_title(date, category_id, title.clone()).execute(db_session).await?;
      
         // find by local secondary index
         let posts: CharybdisModelStream<Post> = Post::find_by_date_and_title(date, title.clone()).execute(db_session).await?;
         let post: Post = Post::find_first_by_date_and_title(date, title.clone()).execute(db_session).await?;
         let post: Option<Post> = Post::maybe_find_first_by_date_and_title(date, title.clone()).execute(db_session).await?;
  
        // find by global secondary index
        let posts: CharybdisModelStream<Post> = Post::find_by_category_id(category_id).execute(db_session).await?;
        let post: Post = Post::find_first_by_category_id(category_id).execute(db_session).await?;
        let post: Option<Post> = Post::maybe_find_first_by_category_id(category_id).execute(db_session).await?;
      
        Ok(())
      }
  }
  ```

- ### Custom filtering:
  Lets use our `Post` model as an example:
    ```rust 
    #[charybdis_model(
        table_name = posts, 
        partition_keys = [category_id], 
        clustering_keys = [date, title],
        global_secondary_indexes = []
    )]
    pub struct Post {...}
    ```
  We get automatically generated `find_post!` macro that follows convention `find_<struct_name>!`.
  It can be used to create custom queries.

  Following will return stream of `Post` models, and query will be constructed at compile time
  as `&'static str`.

    ```rust
    // automatically generated macro rule
    let posts = find_post!(
        "category_id in ? AND date > ?",
        (categor_vec, date)
    ).execute(session).await?;
    ```

  We can also use `find_first_post!` macro to get single result:
    ```rust
    let post = find_first_post!(
        "category_id in ? AND date > ? LIMIT 1",
        (date, categor_vec)
    ).execute(session).await?;
    ```

  If we just need the `Query` and not the result, we can use `find_post_query!` macro:
    ```rust
    let query = find_post_query!(
        "date = ? AND category_id in ?",
        (date, categor_vec)
    ```

## Update

- ```rust
  let user = User::from_json(json);
  
  user.username = "scylla".to_string();
  user.email = "some@email.com";
  
  user.update().execute(&session).await;
  ```
- ### Collection:
    - Let's use our `User` model as an example:
      ```rust
      #[charybdis_model(
          table_name = users,
          partition_keys = [id],
          clustering_keys = [],
      )]
      pub struct User {
          id: Uuid,
          tags: Set<Text>,
          post_ids: List<Uuid>,
      }
      ```
    - `push_to_<field_name>` and `pull_from_<field_name>` methods are generated for each collection
      field.
       ```rust
       let user: User;
   
       user.push_tags(vec![tag]).execute(&session).await;
       user.pull_tags(vec![tag]).execute(&session).await;
   
       user.push_post_ids(vec![tag]).execute(&session).await;
       user.pull_post_ids(vec![tag]).execute(&session).await;
       ```
- ### Counter
    - Let's define post_counter model:
      ```rust
      #[charybdis_model(
          table_name = post_counters,
          partition_keys = [id],
          clustering_keys = [],
      )]
      pub struct PostCounter {
          id: Uuid,
          likes: Counter,
          comments: Counter,
      }
      ```
    - We can use `increment_<field_name>` and `decrement_<field_name>` methods to update counter
      fields.
      ```rust
      let post_counter: PostCounter;
      post_counter.increment_likes(1).execute(&session).await;
      post_counter.decrement_likes(1).execute(&session).await;
      
      post_counter.increment_comments(1).execute(&session).await;
      post_counter.decrement_comments(1).execute(&session).await;
      ```

## Delete

- ```rust 
  let user = User::from_json(json);

  user.delete().execute(&session).await;
  ```

- ### Macro generated delete helpers
  Lets use our `Post` model as an example:
  ```rust
  #[charybdis_model(
      table_name = posts,
      partition_keys = [date],
      clustering_keys = [categogry_id, title],
      global_secondary_indexes = [])
  ]
  pub struct Post {
      date: Date,
      category_id: Uuid,
      title: Text,
      id: Uuid,
      ...
  }
  ```
  We have macro generated functions for up to 3 fields from primary key.

  ```rust
  Post::delete_by_date(date: Date).execute(&session).await?;
  Post::delete_by_date_and_category_id(date: Date, category_id: Uuid).execute(&session).await?;
  Post::delete_by_date_and_category_id_and_title(date: Date, category_id: Uuid, title: Text).execute(&session).await?;
  ```

- ### Custom delete queries
  We can use `delete_post!` macro to create custom delete queries.
    ```rust
    delete_post!("date = ? AND category_id in ?", (date, category_vec)).execute(&session).await?
    ```

## Configuration

Every operation returns `CharybdisQuery` that can be configured before execution with method
chaining.

```rust
let user: User = User::find_by_id(id)
    .consistency(Consistency::One)
    .timeout(Some(Duration::from_secs(5)))
    .execute(&app.session)
    .await?;
    
let result: QueryResult = user.update().consistency(Consistency::One).execute(&session).await?;
```

Supported configuration options:

- `consistency`
- `serial_consistency`
- `timestamp`
- `timeout`
- `page_size`
- `timestamp`

## Batch

`CharybdisModelBatch` operations are used to perform multiple operations in a single batch.

- ### Batch Operations

  ```rust
  let users: Vec<User>;
  let batch = User::batch();
  
  // inserts
  batch.append_inserts(users);
  
  // or updates
  batch.append_updates(users);
  
  // or deletes
  batch.append_deletes(users);
  
  batch.execute(&session).await?;
  ```


- ### Chunked Batch Operations

  Chunked batch operations are used to operate on large amount of data in chunks.
  ```rust
    let users: Vec<User>;
    let chunk_size = 100;
  
    User::batch().chunked_inserts(&session, users, chunk_size).await?;
    User::batch().chunked_updates(&session, users, chunk_size).await?;
    User::batch().chunked_deletes(&session, users, chunk_size).await?;
  ```

- ### Batch Configuration
  Batch operations can be configured before execution with method chaining.
  ```rust
  let batch = User::batch()
      .consistency(Consistency::One)
      .retry_policy(Some(Arc::new(DefaultRetryPolicy::new())))
      .chunked_inserts(&session, users, 100)
      .await?;
  ```
  We could also use method chaining to append operations to batch:
  ```rust
  let batch = User::batch()
      .consistency(Consistency::One)
      .retry_policy(Some(Arc::new(DefaultRetryPolicy::new())))
      .append_update(&user_1)
      .append_update(&user_2)
      .execute(data.db_session())
      .await?;
  ```

- ### Statements Batch
  We can use batch statements to perform collection operations in batch:
    ```rust
    let batch = User::batch();
    let users: Vec<User>;
    
    for user in users {
        batch.append_statement(User::PUSH_TAGS_QUERY, (vec![tag], user.id));
    }
    
    batch.execute(&session).await;
    ```

## Partial Model:

- Use auto generated `partial_<model>!` macro to run operations on subset of the model fields.
  This macro generates a new struct with same structure as the original model, but only with
  provided fields.
  Macro is automatically generated by `#[charybdis_model]`.
  It follows convention `partial_<struct_name>!`.

  ```rust
  // auto-generated macro - available in crate::models::user
  partial_user!(UpdateUsernameUser, id, username);
  ```
  Now we have new struct `UpdateUsernameUser` that is equivalent to `User` model, but only with `id`
  and `username`
  fields.
  ```rust
  let mut update_user_username = UpdateUsernameUser {
      id,
      username: "updated_username".to_string(),
  };
  
  update_user_username.update().execute(&session).await?;
  ```
- ### Partial Model Considerations:
    1) `partial_<model>` requires `#[derive(Default)]` on original model
    2) `partial_<model>` require complete primary key in definition
    3) All derives that are defined bellow `#charybdis_model` macro will be automatically added to
       partial model.
    4) `partial_<model>` struct implements same field attributes as original model,
       so if we have `#[serde(rename = "rootId")]` on original model field, it will be present on
       partial model field.

- ### As Native
  In case we need to run operations on native model, we can use `as_native` method:
  ```rust 
  let native_user: User = update_user_username.as_native().find_by_primary_key().execute(&session).await?;
  // action that requires native model
  authorize_user(&native_user);
  ```
  `as_native` works by returning new instance of native model with fields from partial model.
  For other fields it uses default values.


- Recommended naming convention is `Purpose` + `Original Struct Name`.
  E.g: `UpdateAdresssUser`, `UpdateDescriptionPost`.

## Callbacks

Callbacks are convenient way to run additional logic on model before or after certain operations.
E.g.

- we can use `before_insert` to set default values and/or validate model before insert.
- we can use `after_update` to update other data sources, e.g. elastic search.

### Implementation:

1) Let's say we define custom extension that will be used to
   update elastic document on every post update:
    ```rust
    pub struct AppExtensions {
        pub elastic_client: ElasticClient,
    }
    ```
2) Now we can implement Callback that will utilize this extension:
    ```rust
    #[charybdis_model(...)]
    pub struct Post {}
    
    impl ExtCallbacks for Post {
        type Extention = AppExtensions;
        type Error = AppError; // From<CharybdisError>
        
       // use before_insert to set default values
        async fn before_insert(
            &mut self,
            _session: &CachingSession,
            extension: &AppExtensions,
        ) -> Result<(), CustomError> {
            self.id = Uuid::new_v4();
            self.created_at = Utc::now();
            
            Ok(())
        }
        
        // use before_update to set updated_at
        async fn before_update(
            &mut self,
            _session: &CachingSession,
            extension: &AppExtensions,
        ) -> Result<(), CustomError> {
            self.updated_at = Utc::now();
            
            Ok(())
        }
    
        // use after_update to update elastic document
        async fn after_update(
            &mut self,
            _session: &CachingSession,
            extension: &AppExtensions,
        ) -> Result<(), CustomError> {
            extension.elastic_client.update(...).await?;
    
            Ok(())
        }
        
        // use after_delete to delete elastic document
        async fn after_delete(
            &mut self,
            _session: &CachingSession,
            extension: &AppExtensions,
        ) -> Result<(), CustomError> {
            extension.elastic_client.delete(...).await?;
    
            Ok(())
        }
    }
    ```

- ### Possible Callbacks:
    - `before_insert`
    - `before_update`
    - `before_delete`
    - `after_insert`
    - `after_update`
    - `after_delete`
- ### Triggering Callbacks
  In order to trigger callback we use `<operation>_cb`.
  method: `insert_cb`, `update_cb`, `delete_cb` according traits.
  This enables us to have clear distinction between `insert` and insert with
  callbacks (`insert_cb`).
  Just as on main operation, we can configure callback operation query before execution.
  ```rust
   use charybdis::operations::{DeleteWithCallbacks, InsertWithCallbacks, UpdateWithCallbacks};
  
   post.insert_cb(app_extensions).execute(&session).await;
   post.update_cb(app_extensions).execute(&session).await;
   post.delete_cb(app_extensions).consistency(Consistency::All).execute(&session).await;
  ```

## Collections

For each collection field, we get following:

- `PUSH_<field_name>_QUERY` static str
- `PULL_<field_name>_QUERY` static str
- `push_<field_name>` method
- `pull_<field_name>` method

1) ### Model:
    ```rust
    #[charybdis_model(
        table_name = users,
        partition_keys = [id],
        clustering_keys = []
    )]
    pub struct User {
        id: Uuid,
        tags: Set<Text>,
        post_ids: List<Uuid>,
        books_by_genre: Map<Text, Frozen<List<Text>>>,
    }
    ```
2) ### Generated Collection Queries:
    ```rust
    User::PUSH_TAGS_QUERY;
    User::PULL_TAGS_QUERY;
    
    User::PUSH_POST_IDS_QUERY;
    User::PULL_POST_IDS_QUERY;
    ```

   Generated query will expect value as first bind value and primary key fields as next bind values.

    ```rust
    impl User {
      const PUSH_TAGS_QUERY: &'static str = "UPDATE users SET tags = tags + ? WHERE id = ?";
      const PULL_TAGS_QUERY: &'static str = "UPDATE users SET tags = tags - ? WHERE id = ?";.
       
      const PUSH_POST_IDS_QUERY: &'static str = "UPDATE users SET post_ids = post_ids + ? WHERE id = ?";
      const PULL_POST_IDS_QUERY: &'static str = "UPDATE users SET post_ids = post_ids - ? WHERE id = ?";
    
      const PUSH_BOOKS_BY_GENRE_QUERY: &'static str = "UPDATE users SET books_by_genre = books_by_genre + ? WHERE id = ?";
      const PULL_BOOKS_BY_GENRE_QUERY: &'static str = "UPDATE users SET books_by_genre = books_by_genre - ? WHERE id = ?";
    }
    ```

   Now we could use this constant within Batch operations.

    ```rust
    let batch = User::batch();
    let users: Vec<User>;
    
    for user in users {
        batch.append_statement(User::PUSH_TAGS_QUERY, (vec![tag], user.id));
    }
    
    batch.execute(&session).await;
    ```

3) ### Generated Collection Methods:
   `push_to_<field_name>` and `pull_from_<field_name>` methods are generated for each collection
   field.

    ```rust
    let user: User::new();
    
    user.push_tags(tags: HashSet<T>).execute(&session).await;
    user.pull_tags(tags: HashSet<T>).execute(&session).await;
    
    
    user.push_post_ids(ids: Vec<T>).execute(&session).await;
    user.pull_post_ids(ids: Vec<T>).execute(&session).await;
    
    user.push_books_by_genre(map: HashMap<K, V>).execute(&session).await;
    user.pull_books_by_genre(map: HashMap<K, V>).execute(&session).await;
    ```

## Ignored fields

We can ignore fields by using `#[charybdis(ignore)]` attribute:

```rust
#[charybdis_model(...)]
pub struct User {
    id: Uuid,
    #[charybdis(ignore)]
    organization: Option<Organization>,
}
```

So field `organization` will be ignored in all operations and
default value will be used when deserializing from other data sources.
It can be used to hold data that is not persisted in database.
