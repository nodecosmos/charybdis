## Automatic migration Tool:

<a name="automatic-migration"></a>
`charybdis-migrate` tool that enables automatic migration to database without need to write
migrations by hand. It expects `src/models` files and generates migrations based on differences
between model definitions and database.

## Announcements:

- ### Breaking changes:
    - As of **0.4.3** version`local_secondary_indexes` are now defined as list of fields. Partition key part is derived
      from `partition_keys` part of macro declaration and each element in array will result with new local index.

### Installation

```bash
  cargo install charybdis-migrate
```

### Usage

```bash
migrate --hosts <host> --keyspace <your_keyspace> --drop-and-replace (optional)
```

## Automatic migration

* <a name="automatic-migration"></a>
  `charybdis-migrate` enables automatic migration to database without need to write migrations by hand.
  It expects `src/models` files and generates migrations based on differences between model definitions and database.
  It supports following operations:
    - Create new tables
    - Create new columns
    - Drop columns
    - Change field types (drop and recreate column `--drop-and-replace` flag)
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

      Model dropping is not added. If you don't define model within `src/model` dir
      it will leave db structure as it is.

* ### Running migration

  ⚠️ If you are working with **existing** datasets, before running migration you need to make sure that your **model
  **
  definitions structure matches the database in respect to table names, column names, column types, partition keys,
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

### Define Tables

- Declare model as a struct within `src/models` dir:
  ```rust
  // src/models/user.rs
  use charybdis::macros::charybdis_model;
  use charybdis::types::{Text, Timestamp, Uuid};
  
  #[charybdis_model(
      table_name = users,
      partition_keys = [id],
      clustering_keys = [],
      global_secondary_indexes = [username],
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

- `src/models/udts`
  ```rust
  // src/models/udts/address.rs
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

### Define Materialized Views

- `src/models/materialized_views`

  ```rust
  // src/models/materialized_views/users_by_username.rs
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
