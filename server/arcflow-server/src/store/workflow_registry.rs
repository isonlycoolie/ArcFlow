//! PostgreSQL persistence for versioned workflow definitions.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PublishedWorkflow {
    pub name: String,
    pub version: String,
    pub schema_hash: String,
    pub definition_json: Value,
    pub published_by: Option<String>,
    pub published_at: DateTime<Utc>,
    pub deprecated: bool,
}

pub struct WorkflowRegistryStore {
    pool: PgPool,
}

impl WorkflowRegistryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn publish(
        &self,
        name: &str,
        version: &str,
        schema_hash: &str,
        definition_json: Value,
        published_by: Option<&str>,
    ) -> Result<PublishedWorkflow, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO arcflow_workflows
             (name, version, schema_hash, definition_json, published_by)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (name, version) DO UPDATE SET
                 schema_hash = EXCLUDED.schema_hash,
                 definition_json = EXCLUDED.definition_json,
                 published_by = EXCLUDED.published_by,
                 published_at = CASE
                     WHEN arcflow_workflows.schema_hash IS DISTINCT FROM EXCLUDED.schema_hash
                     THEN NOW()
                     ELSE arcflow_workflows.published_at
                 END
             RETURNING name, version, schema_hash, definition_json, published_by, published_at, deprecated",
        )
        .bind(name)
        .bind(version)
        .bind(schema_hash)
        .bind(definition_json)
        .bind(published_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(map_row(&row))
    }

    pub async fn get(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<PublishedWorkflow>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT name, version, schema_hash, definition_json, published_by, published_at, deprecated
             FROM arcflow_workflows WHERE name = $1 AND version = $2",
        )
        .bind(name)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| map_row(&r)))
    }

    pub async fn list_versions(&self, name: &str) -> Result<Vec<String>, sqlx::Error> {
        use sqlx::Row;
        let rows = sqlx::query(
            "SELECT version FROM arcflow_workflows WHERE name = $1 AND deprecated = FALSE ORDER BY published_at DESC",
        )
        .bind(name)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.get("version")).collect())
    }

    pub async fn set_alias(
        &self,
        name: &str,
        alias: &str,
        version: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO arcflow_workflow_aliases (name, alias, version)
             VALUES ($1, $2, $3)
             ON CONFLICT (name, alias) DO UPDATE SET version = EXCLUDED.version",
        )
        .bind(name)
        .bind(alias)
        .bind(version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_alias_version(
        &self,
        name: &str,
        alias: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT version FROM arcflow_workflow_aliases WHERE name = $1 AND alias = $2",
        )
        .bind(name)
        .bind(alias)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.get("version")))
    }
}

fn map_row(row: &sqlx::postgres::PgRow) -> PublishedWorkflow {
    use sqlx::Row;
    PublishedWorkflow {
        name: row.get("name"),
        version: row.get("version"),
        schema_hash: row.get("schema_hash"),
        definition_json: row.get("definition_json"),
        published_by: row.get("published_by"),
        published_at: row.get("published_at"),
        deprecated: row.get("deprecated"),
    }
}
