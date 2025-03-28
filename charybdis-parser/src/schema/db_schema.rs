use std::collections::HashMap;

use colored::Colorize;
use scylla::client::session::Session;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use crate::errors::DbSchemaParserError;
use crate::schema::secondary_indexes::{IndexTarget, SecondaryIndex};
use crate::schema::{SchemaObject, SchemaObjects};

#[derive(Debug, Serialize, Deserialize)]
pub struct DbSchema {
    pub tables: SchemaObjects,
    pub udts: SchemaObjects,
    pub materialized_views: SchemaObjects,
    pub keyspace_name: String,
}

/**
 * DbSchema is a singleton that contains the current state of the database schema.
 * It is populated by the get_current_schema() function.
 * It is used to compare the current state to the desired state of the database schema.
 */
impl DbSchema {
    pub async fn new(session: &Session, keyspace_name: String) -> DbSchema {
        let mut current_schema = DbSchema {
            tables: HashMap::new(),
            udts: HashMap::new(),
            materialized_views: HashMap::new(),
            keyspace_name,
        };

        current_schema
            .get_tables_from_system_schema(session)
            .await
            .map_err(|e| {
                println!(
                    "{}\n",
                    format!("Error getting tables from system_schema: {}", e)
                        .bright_red()
                        .bold()
                );
                e
            })
            .unwrap();

        current_schema
            .get_udts_from_system_schema(session)
            .await
            .map_err(|e| {
                println!(
                    "{}\n",
                    format!("Error getting udts from system_schema: {}", e)
                        .bright_red()
                        .bold()
                );
                e
            })
            .unwrap();

        current_schema
            .get_mvs_from_system_schema(session)
            .await
            .map_err(|e| {
                println!(
                    "{}\n",
                    format!("Error getting materialized views from system_schema: {}", e)
                        .bright_red()
                        .bold()
                );
                e
            })
            .unwrap();

        current_schema
    }

    async fn get_tables_from_system_schema(&mut self, session: &Session) -> Result<(), DbSchemaParserError> {
        // get tables as a HashMap of column_name => column_type
        // Parse row as a single column containing an int value
        let cql = r#"
            SELECT table_name
            FROM system_schema.tables
            WHERE keyspace_name = ?
            ALLOW FILTERING
        "#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name,))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String,)>()? {
            let table_name = row?.0;
            self.tables.insert(table_name.clone(), SchemaObject::new());
            self.populate_table_columns(&table_name, session).await?;
            self.populate_table_partition_keys(&table_name, session).await?;
            self.populate_table_clustering_keys(&table_name, session).await?;
            self.populate_table_secondary_indexes(&table_name, session).await?;
        }

        Ok(())
    }

    async fn populate_table_columns(
        &mut self,
        table_name: &String,
        session: &Session,
    ) -> Result<(), DbSchemaParserError> {
        // get columns and types for provided table
        let cql = r#"
            SELECT
                column_name, type
            FROM system_schema.columns
            WHERE keyspace_name = ? 
                AND table_name = ?
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name, &table_name))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String, String)>()? {
            let str_value: (String, String) = row?;
            self.tables
                .get_mut(table_name)
                .unwrap()
                .push_field(str_value.0, str_value.1, false);
        }

        Ok(())
    }

    async fn populate_table_partition_keys(
        &mut self,
        table_name: &String,
        session: &Session,
    ) -> Result<(), DbSchemaParserError> {
        // get partition keys for provided table
        let cql = r#"
            SELECT column_name
            FROM system_schema.columns
            WHERE keyspace_name = ?
                AND table_name = ?
                AND kind = 'partition_key'
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name, &table_name))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String,)>()? {
            let str_value: (String,) = row?;
            self.tables
                .get_mut(table_name)
                .unwrap()
                .partition_keys
                .push(str_value.0);
        }

        Ok(())
    }

    async fn populate_table_clustering_keys(
        &mut self,
        table_name: &String,
        session: &Session,
    ) -> Result<(), DbSchemaParserError> {
        // get partition keys for provided table
        let cql = r#"
            SELECT column_name
            FROM system_schema.columns
            WHERE keyspace_name = ?
                AND table_name = ?
                AND kind = 'clustering'
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name, &table_name))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String,)>()? {
            let str_value: (String,) = row?;
            self.tables
                .get_mut(table_name)
                .unwrap()
                .clustering_keys
                .push(str_value.0);
        }

        Ok(())
    }

    async fn populate_table_secondary_indexes(
        &mut self,
        table_name: &String,
        session: &Session,
    ) -> Result<(), DbSchemaParserError> {
        // get partition keys for provided table
        let cql = r#"
            SELECT index_name, options
            FROM system_schema.indexes
            WHERE keyspace_name = ?
                AND table_name = ?
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name, &table_name))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String, SecondaryIndex)>()? {
            let value: (String, SecondaryIndex) = row?;
            let table_schema = self.tables.get_mut(table_name).unwrap();

            let index_name = value.0;
            let index_target = value.1;

            match index_target.target {
                IndexTarget::GlobalSecondaryIndex(target) => {
                    table_schema.global_secondary_indexes.push((index_name, target));
                }
                IndexTarget::LocalSecondaryIndex(target) => {
                    table_schema.local_secondary_indexes.push((index_name, target));
                }
            }
        }

        Ok(())
    }

    async fn get_udts_from_system_schema(&mut self, session: &Session) -> Result<(), DbSchemaParserError> {
        // get tables as a HashMap of column_name => column_type
        // Parse row as a single column containing an int value
        let cql = r#"
            SELECT
                type_name,
                field_names,
                field_types
            FROM system_schema.types
            WHERE keyspace_name = ?"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name,))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String, Vec<String>, Vec<String>)>()? {
            let (type_name, field_names, field_types) = row?;

            let mut schema_object = SchemaObject::new();

            for (index, field_name) in field_names.into_iter().enumerate() {
                schema_object.push_field(field_name, field_types[index].clone(), false);
            }

            self.udts.insert(type_name.to_lowercase(), schema_object);
        }

        Ok(())
    }

    async fn get_mvs_from_system_schema(&mut self, session: &Session) -> Result<(), DbSchemaParserError> {
        // get tables as a HashMap of column_name => column_type
        let cql = r#"
            SELECT view_name
            FROM system_schema.views
            WHERE keyspace_name = ?
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name,))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String,)>()? {
            let view_name = row?.0;
            self.materialized_views.insert(view_name.clone(), SchemaObject::new());
            self.populate_mv_columns(&view_name, session).await?;
            self.populate_mv_partition_key(&view_name, session).await?;
            self.populate_mv_clustering_keys(&view_name, session).await?;
        }

        Ok(())
    }

    async fn populate_mv_columns(&mut self, view_name: &String, session: &Session) -> Result<(), DbSchemaParserError> {
        // get columns and types for views
        let cql = r#"
            SELECT column_name, type
            FROM system_schema.columns
            WHERE keyspace_name = ?
                AND table_name = ?
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name, &view_name))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String, String)>()? {
            let str_value: (String, String) = row?;
            self.materialized_views
                .get_mut(view_name)
                .unwrap()
                .push_field(str_value.0, str_value.1, false);
        }

        Ok(())
    }

    async fn populate_mv_partition_key(
        &mut self,
        view_name: &String,
        session: &Session,
    ) -> Result<(), DbSchemaParserError> {
        let cql = r#"
            SELECT column_name
            FROM system_schema.columns
            WHERE keyspace_name = ?
                AND table_name = ?
                AND kind = 'partition_key'
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name, &view_name))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String,)>()? {
            let str_value: (String,) = row?;
            self.materialized_views
                .get_mut(view_name)
                .expect("Expected a materialized view")
                .partition_keys
                .push(str_value.0);
        }

        Ok(())
    }

    async fn populate_mv_clustering_keys(
        &mut self,
        view_name: &String,
        session: &Session,
    ) -> Result<(), DbSchemaParserError> {
        let cql = r#"
            SELECT column_name
            FROM system_schema.columns
            WHERE keyspace_name = ?
                AND table_name = ?
                AND kind = 'clustering'
            ALLOW FILTERING"#;

        let result = session
            .query_unpaged(cql, (&self.keyspace_name, &view_name))
            .await?
            .into_rows_result()?;

        for row in result.rows::<(String,)>()? {
            let str_value: (String,) = row?;
            self.materialized_views
                .get_mut(view_name)
                .expect("Expected a materialized view")
                .clustering_keys
                .push(str_value.0);
        }

        Ok(())
    }

    pub fn get_current_schema_as_json(&self) -> String {
        to_string_pretty(&self).unwrap_or_else(|e| {
            panic!("Error serializing schema to json: {}", e);
        })
    }

    pub fn write_schema_to_json(&self, current_dir: &str) {
        let json = self.get_current_schema_as_json();

        let path = current_dir.to_string() + "/current_schema.json";

        std::fs::write(path, json).unwrap_or_else(|e| {
            panic!("Error writing schema to json: {}", e);
        });
    }
}
