//! PostgreSQL schema migrations (`sqlx` embedded).

use sqlx::postgres::PgPool;
use sqlx::migrate::MigrateError;

/// Embedded migrations from `runtime/arcflow-core/migrations/`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Applies all pending migrations.
pub async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await
}

/// Latest embedded migration version (unix millis prefix in filenames).
pub fn head_version() -> i64 {
    MIGRATOR
        .migrations
        .iter()
        .map(|m| m.version)
        .max()
        .unwrap_or(0)
}

/// Returns true when applied schema is behind embedded head.
pub async fn pending(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let head = head_version();
    let applied: Result<i64, sqlx::Error> =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
            .fetch_one(pool)
            .await;
    match applied {
        Ok(v) => Ok(v < head),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("42P01") => Ok(true),
        Err(e) => Err(e),
    }
}

/// Returns true when applied schema matches embedded head (no drift check).
pub async fn at_head(pool: &PgPool) -> Result<bool, sqlx::Error> {
    Ok(!pending(pool).await?)
}
