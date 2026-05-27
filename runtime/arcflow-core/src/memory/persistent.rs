//! PostgreSQL-backed persistent memory (lazy connect).

use std::env;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use super::error::MemoryError;
use super::namespace::durable_key;

/// Durable key-value store in PostgreSQL.
pub struct PersistentMemory {
    pool: Option<PgPool>,
}

impl Default for PersistentMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistentMemory {
    /// Not connected until first use.
    pub fn new() -> Self {
        Self { pool: None }
    }

    async fn pool(&mut self) -> Result<&PgPool, MemoryError> {
        if self.pool.is_none() {
            let url = env::var("ARCFLOW_POSTGRESQL_URL").map_err(|_| {
                MemoryError::InfrastructureUnavailable {
                    backend: "postgresql".into(),
                    suggestion: "Set ARCFLOW_POSTGRESQL_URL and start Postgres.".into(),
                }
            })?;
            let pool = PgPoolOptions::new()
                .max_connections(2)
                .connect(&url)
                .await
                .map_err(|e| MemoryError::InfrastructureUnavailable {
                    backend: "postgresql".into(),
                    suggestion: format!("connection failed: {e}"),
                })?;
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS arcflow_memory (
                    storage_key TEXT PRIMARY KEY,
                    value BYTEA NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )",
            )
            .execute(&pool)
            .await
            .map_err(|e| MemoryError::OperationFailed {
                reason: e.to_string(),
            })?;
            self.pool = Some(pool);
        }
        Ok(self.pool.as_ref().expect("pool initialized"))
    }

    /// Upserts a value under namespace.
    pub async fn write(
        &mut self,
        namespace: &str,
        logical_key: &str,
        value: &[u8],
    ) -> Result<(), MemoryError> {
        let key = durable_key(namespace, logical_key);
        let pool = self.pool().await?;
        sqlx::query(
            "INSERT INTO arcflow_memory (storage_key, value) VALUES ($1, $2)
             ON CONFLICT (storage_key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()",
        )
        .bind(&key)
        .bind(value)
        .execute(pool)
        .await
        .map_err(|e| MemoryError::OperationFailed {
            reason: e.to_string(),
        })?;
        Ok(())
    }

    /// Reads a value.
    pub async fn read(
        &mut self,
        namespace: &str,
        logical_key: &str,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        let key = durable_key(namespace, logical_key);
        let pool = self.pool().await?;
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT value FROM arcflow_memory WHERE storage_key = $1",
        )
        .bind(&key)
        .fetch_optional(pool)
        .await
        .map_err(|e| MemoryError::OperationFailed {
            reason: e.to_string(),
        })?;
        Ok(row.map(|r| r.0))
    }
}
