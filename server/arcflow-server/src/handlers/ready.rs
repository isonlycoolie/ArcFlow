use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn ready(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    let version = env!("CARGO_PKG_VERSION");
    let Some(pool) = state.pg_pool.as_ref() else {
        return (
            StatusCode::OK,
            Json(json!({ "status": "ready", "version": version, "postgres": "not_configured" })),
        );
    };
    if let Err(e) = sqlx::query("SELECT 1").execute(pool).await {
        tracing::warn!(error = %e, "ready: postgres ping failed");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "degraded",
                "version": version,
                "reason": "postgres_unavailable"
            })),
        );
    }
    match arcflow_core::migrate::pending(pool).await {
        Ok(true) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "degraded",
                "version": version,
                "reason": "migrations_pending"
            })),
        ),
        Ok(false) => (
            StatusCode::OK,
            Json(json!({ "status": "ready", "version": version })),
        ),
        Err(e) => {
            tracing::warn!(error = %e, "ready: migration check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "version": version,
                    "reason": "migration_check_failed"
                })),
            )
        }
    }
}
