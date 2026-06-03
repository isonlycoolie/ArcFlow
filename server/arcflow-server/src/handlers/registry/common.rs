//! Shared registry handler helpers.

use axum::http::StatusCode;
use std::sync::Arc;

use crate::state::AppState;

pub fn registry_store(
    state: &AppState,
) -> Result<&Arc<crate::store::workflow_registry::WorkflowRegistryStore>, (StatusCode, String)> {
    state.registry.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "[ArcFlow] ARCFLOW_POSTGRESQL_URL is required for workflow registry".into(),
    ))
}

pub fn internal(err: sqlx::Error) -> (StatusCode, String) {
    tracing::warn!(error = %err, "registry database error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "[ArcFlow] Database error".into(),
    )
}

pub fn bad_request(message: impl Into<String>) -> (StatusCode, String) {
    (
        StatusCode::BAD_REQUEST,
        format!("[ArcFlow] {}", message.into()),
    )
}

pub fn not_found(message: &str) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("[ArcFlow] {message}"))
}

pub fn to_definition_response(
    published: crate::store::workflow_registry::PublishedWorkflow,
) -> crate::dto::registry::WorkflowDefinitionResponse {
    crate::dto::registry::WorkflowDefinitionResponse {
        name: published.name,
        version: published.version,
        schema_hash: published.schema_hash,
        definition: published.definition_json,
    }
}
