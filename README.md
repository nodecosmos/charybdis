# Rust ORM for ScyllaDB
### Use monstrous tandem of scylla and charybdis for your next project
⚠️ *WIP*: This project is currently in an experimental stage. It depends on Rust: 1.75.0 which is currently Beta and it's scheduled for stable release on [Dec 28](https://forge.rust-lang.org/)



<img src="https://www.scylladb.com/wp-content/uploads/scylla-opensource-1.png" height="250">

#### Charybdis is a ORM layer on top of `scylla_rust_driver` focused on easy of use and performance

## Usage considerations:
- Provide and expressive API for CRUD & Complex Query operations on model as a whole
- Provide easy way to work with subset of model fields by using automatically generated `partial_<model>!` macro
- Provide easy way to run complex queries by using automatically generated `find_<model>!` macro
- Automatic migration tool that analyzes the `src/model/*.rs` files and runs migrations according to differences between the model definition and database

## Performance consideration:
- It's build by beta release, so it uses builtin support for `async/await` in traits that will be stabilized in Rust `1.75`
- It uses prepared statements (shard/token aware) -> bind values
- It expects `CachingSession` as a session arg for operations
- Queries are macro generated str constants (no concatenation at runtime)
- By using `find_<model>!` macro we can run complex queries that are generated at compile time as `&'static str`
- Although it has expressive API it's thin layer on top of scylla_rust_driver, and it does not introduce any significant overhead


## Table of Contents
- [Charybdis Models](#charybdis-models)
  - [Define Tables](#define-tables)
  - [Define UDTs](#Define-UDT)
  - [Define Materialized Views](#Define-Materialized-Views)
- [Automatic migration with `charybdis-migrate`](#automatic-migration-with-charybdiscmdmigrate)
- [Basic Operations](#basic-operations)
  - [Create](#create)
  - [Find](#find)
    - [Custom filtering](#custom-filtering)
  - [Update](#update)
  - [Delete](#delete)
- [Partial Model Operations](#partial-model-operations)
  - [Considerations](#partial-model-considerations)
- [View Operations](#view-operations)
- [Callbacks](#callbacks)
- [Batch Operations](#batch-operations)
- [As Native](#as-native)
- [Collection queries](#collection-queries)
- [Ignored fields](#ignored-fields)
- [Roadmap](#Roadmap)

## Charybdis Models


### Define Tables

Declare model as a struct within `src/models` dir:
```rust
// src/modles/user.rs
use charybdis_macros::charybdis_model;
use charybdis::types::{Text, Timestamp, Uuid};

#[charybdis_model(
    table_name = users,
    partition_keys = [id],
    clustering_keys = [],
    global_secondary_indexes = [],
    local_secondary_indexes = [],
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
(Note we use `src/models` as automatic migration tool expects that dir)

### Define UDT
Declare udt model as a struct within `src/models/udts` dir:
```rust
// src/models/udts/address.rs
use charybdis_macros::charybdis_udt_model;
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
### Define Materialized Views
Declare view model as a struct within `src/models/materialized_views` dir:

```rust
// src/models/materialized_views/users_by_username.rs
use charybdis_macros::charybdis_view_model;
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
```cassandraql
CREATE MATERIALIZED VIEW IF NOT EXISTS users_by_email
AS SELECT created_at, updated_at, username, email, id
FROM users
WHERE email IS NOT NULL AND id IS NOT NULL
PRIMARY KEY (email, id)
  ```

📝 Primary key fields should not be wrapped in Option<> as they are mandatory.
 
## Automatic migration
<a name="automatic-migration"></a>
`charybdis-migrate` tool that enables automatic migration to database without need to write migrations by hand.
It expects `src/models` files and generates migrations based on differences between model definitions and database.

It supports following operations:
- Create new tables
- Create new columns
- Drop columns
- Create secondary indexes
- Drop secondary indexes
- Create UDTs (`src/models/udts`)
- Create materialized views (`src/models/materialized_views`)
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
    ⚠️ If table exists, table options will result in alter table query that without
    `CLUSTERING ORDER` and `COMPACT STORAGE` options.

🟢 Tables, Types and UDT droppin``g is not added. If you don't define model within `src/model` dir 
it will leave db structure as it is.
```bash
cargo install charybdis-migrate

migrate --hosts <host> --keyspace <your_keyspace>
```

⚠️ If you are working with **existing** datasets, before running migration you need to make sure that your **model** 
definitions structure matches the database in respect to table names, column names, column types, partition keys, 
clustering keys and secondary indexes so you don't alter structure accidentally.
If structure is matched, it will not run any migrations. As mentioned above, 
in case there is no model definition for table, it will **not** drop it. In future, 
we will add `modelize` command that will generate `src/models` files from existing data source.

### Global secondary indexes
They are simply defined as array of strings:
```rust
#[charybdis_model(
    table_name = users,
    partition_keys = [id],
    clustering_keys = [],
    global_secondary_indexes = ["username"]
)]
```
### Local secondary Indexes

They are defined as array of tuples
- first element is array of partition keys
- second element is array of clustering keys
```rust
#[charybdis_model(
    table_name = menus,
    partition_keys = [location],
    clustering_keys = [name, price, dish_type],
    global_secondary_indexes = [],
    local_secondary_indexes = [
        ([location], [dish_type])
    ]
)]
```
resulting query will be: `CREATE INDEX ON menus((location), dish_type);`

## Basic Operations:
For each operation you need to bring respective trait into scope. They are defined
in `charybdis::operations` module.

### Create

```rust
use charybdis::{CachingSession, Insert};

#[tokio::main]
async fn main() {
  let session: &CachingSession; // init session
  
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
  user.insert(&session).await;
}
```

## Find

#### Find by primary key
This is preferred way to query data rather then
find_by_primary_key_val `associated fun`, as it will automatically provide correct order
based on primary key definition.
```rust
  let user = User {id, ..Default::default()};
  let user = user.find_by_primary_key(&session).await?;
```
#### Find by partition key

```rust
  let users =  User {id, ..Default::default()}.find_by_partition_key(&session).await;
```

#### Macro generated find helpers
Lets say we have model:
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
    title: String,
    id: Uuid,
    ...
}
```

```rust
Post::find_by_date(session: &Session, date: Date) -> Result<CharybdisModelStream<Post>, CharybdisError>
Post::find_by_date_and_category_id(session: &Session, date: Date, category_id: Uuid) ->  Result<CharybdisModelStream<Post>, CharybdisError>
Post::find_by_date_and_category_id_and_title(session: &Session, date: Date, category_id: Uuid, title: String) -> Result<Post, CharybdisError>
```
We have macro generated  functions for up to 3 fields from primary key. Note that if **complete** 
primary key is provided, we get single typed result.


### Custom filtering:
Let's say we have a model:
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


Following will return stream of `Post` models, and query will be constructed at compile time as `&'static str`.

```rust
// automatically generated macro rule
let posts = find_post!(
    session,
    "category_id in ? AND date > ?",
    (categor_vec, date])
).await?;
```

We can also use `find_first_post!` macro to get single result:
```rust
let post = find_first_post!(
    session,
    "category_id in ? AND date > ? LIMIT 1",
    (date, categor_vec]
).await?;
```

If we just need the `Query` and not the result, we can use `find_post_query!` macro:
```rust
let query = find_post_query!(
    "date = ? AND category_id in ?",
    (date, categor_vec])
```

## Update
```rust
let user = User::from_json(json);

user.username = "scylla".to_string();
user.email = "some@email.com";

user.update(&session).await;
```

## Delete
```rust 
  let user = User::from_json(json);

  user.delete(&session).await;
```

### Macro generated delete helpers
Lets say we have model:
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
    title: String,
    id: Uuid,
    ...
}
```
We have macro generated  functions for up to 3 fields from primary key.

```rust
Post::delete_by_date(session: &Session, date: Date);
Post::delete_by_date_and_category_id(session: &Session, date: Date, category_id: Uuid);
Post::delete_by_date_and_category_id_and_title(session: &Session, date: Date, category_id: Uuid, title: String);
```


## Partial Model Operations:
Use auto generated `partial_<model>!` macro to run operations on subset of the model fields.
This macro generates a new struct with same structure as the original model, but only with provided fields.
Macro is automatically generated by `#[charybdis_model]`.
It follows convention `partial_<struct_name>!`.

```rust
// auto-generated macro - available in crate::models::user
partial_user!(UpdateUsernameUser, id, username);

let id = Uuid::new_v4();
let user = UpdateUsernameUser { id, username: "scylla".to_string() };

// we can have same operations as on base model
// INSERT into users (id, username) VALUES (?, ?)
user.insert(&session).await;

// UPDATE users SET username = ? WHERE id = ?
user.update(&session).await;

// DELETE FROM users WHERE id = ?
user.delete(&session).await;

// get partial PartUser
let partial_user = user.find_by_primary_key(&:session).await?;

// get native user model by primary key
let user = user.as_native().find_by_primary_key(&session).await?;
```


### Partial Model Considerations:
1) `partial_<model>` require complete primary key in definition

2) All derives that are defined bellow `#charybdis_model` macro will be automatically added to partial model.

3) `partial_<model>` struct implements same field attributes as original model, 
    so if we have `#[serde(rename = "rootId")]` on original model field, it will be present on partial model field.

Recommended naming convention is `Purpose` + `Original Struct Name`. E.g: 
`UpdateAdresssUser`, `UpdateDescriptionPost`. 

## Callbacks
We can define callbacks that will be executed before and after certain operations.
Note that callbacks returns custom error class that implements `From<CharybdisError>`.

```rust
use charybdis::*;

#[charybdis_model(
    table_name = organizations, 
    partition_keys = [id], 
    clustering_keys = [],
    global_secondary_indexes = [name]
)]
pub struct Organization {
    ...
}

impl Organization {
  pub async fn find_by_name(&self, session: &CachingSession) -> Option<Organization> {
    find_first_organization!(session, "name = ?", (&self.name,)).await.ok()
  }
}

impl Callbacks for Organization {
  type Error = CustomError;

  async fn before_insert(&self, session: &CachingSession) -> Result<(), CustomError> {
    if self.find_by_name(session).await.is_some() {
      return Err(CustomError::ValidationError((
        "name".to_string(),
        "is taken".to_string(),
      )));
    }

    Ok(())
  }
}
```
Possible callbacks:
- `before_insert`
- `after_insert`
- `before_update`
- `after_update`
- `before_delete`
- `after_delete`

⚠️ In order to trigger callback, instead of calling `insert` method on model, we can call 
`insert_cb`. This enables us to have clear distinction between insert and insert with callbacks.
```rust
let post = Post::from_json(json);
let res = post.insert_cb(&session).await;
match res {
        Ok(_) => println!("success"),
        Err(e) => match e {
            CharybdisError::ValidationError((field, reason)) => {
                println!("validation error: {} {}", field, reason)
            }
            _ => println!("error: {:?}", e),
        },
    }
```

## ExtensionCallbacks
We can also define callbacks that will be given custom extension if needed.

Let's say we define custom extension that will be used to 
update elastic document on every post update:
```rust
pub struct CustomExtension {
    pub elastic_client: ElasticClient,
}
```

We can define `after_update` callback on `Post`  
that has custom extension as type:
```rust
#[charybdis_model(...)]
pub struct Post {}

impl ExtCallbacks for Post {
    type Extention = CustomExtension;
    type Error = CustomError;

    async fn after_update(
        &mut self,
        _db_session: &CachingSession,
        extension: &CustomExtension,
    ) -> Result<(), CustomError> {
        extension.elastic_client.update(...).await?;

        Ok(())
    }
}
```

So to trigger callback we use same `update_cb` method:
```rust
let post = Post::from_json(json);
let res = post.update_cb(&session, custom_extensions).await;
```
Note that CustomError has to implement `From<CharybdisError>`.

## Batch Operations

For batched operations we can make use of `CharybdisModelBatch`.

```rust
let mut batch = CharybdisModelBatch::new();
let users: Vec<User> = Vec::from_json(json);

// inserts
batch.append_inserts(users);

// or updates
batch.append_updates(users);

// or deletes
batch.append_deletes(users);

batch.execute(&session).await;
```

It also supports chunked batch operations
```rust
chunk_size = 100;
CharybdisModelBatch::chunked_inserts(&session, users, chunk_size).await?;
```

## As Native
In case we need to run operations on native model, we can use `as_native` method:
```rust
partial_user!(UpdateUser, id, username);

let mut update_user_username = UpdateUser {
    id,
    username: "updated_username".to_string(),
};

let native_user: User = update_user_username.as_native().find_by_primary_key(&session).await?;

// action that requires native model
authorize_user(&native_user);
```
`as_native` works by returning new instance of native model with fields from partial model.
For other fields it uses default values.

## Collections
For every field that is defined with `List<T> `type or `Set<T>`, we get following:
- `PUSH_<field_name>_QUERY` static str
- `PULL_<field_name>_QUERY` static str
- `push_<field_name>` method
- `pull_<field_name>` method

```rust
pub struct User {
    id: Uuid,
    tags: Set<String>,
    post_ids: List<Uuid>,
}

let query = User::PUSH_TAGS_QUERY;
execute(query, (vec![tag], &user.id)).await;

let query = User::PULL_POST_IDS_QUERY;
execute(query, (post_ids_vec, &user.id)).await;
```

Methods take session and value as arguments:
```rust
let user = User::from_json(json);
user.push_tags(&session, vec![tag]).await;
user.pull_post_ids(&session, post_ids_vec).await;
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

## Roadmap:
- [ ] Add tests
- [ ] Write `modelize` command to generate `src/models/*` structs from existing database
- [ ] Add --drop flag to migrate command to drop tables, types and UDTs if they are not defined in 
  `src/models`
