//! Admin knowledge ingest handler.

use std::sync::Arc;

use arcflow_core::memory::VectorMemory;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::dto::sites::{IngestKnowledgeRequest, IngestKnowledgeResponse};
use crate::state::AppState;

pub async fn ingest_knowledge(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
    Json(body): Json<IngestKnowledgeRequest>,
) -> Result<Json<IngestKnowledgeResponse>, (StatusCode, String)> {
    let store = state.sites.as_ref().ok_or(service_unavailable())?;
    let site = store
        .get(&site_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("site not found"))?;

    if body.text.trim().is_empty() {
        return Err(bad_request("text must be non-empty"));
    }
    let key = body
        .key
        .as_deref()
        .filter(|k| !k.is_empty())
        .unwrap_or("doc");

    let mut memory = VectorMemory::new();
    let chunks = memory
        .write_document(&site.kb_namespace, key, body.text.trim())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(IngestKnowledgeResponse {
        chunks_ingested: chunks,
        namespace: site.kb_namespace,
    }))
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.into())
}

fn not_found(msg: &str) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, msg.into())
}

fn service_unavailable() -> (StatusCode, String) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "[ArcFlow] ARCFLOW_POSTGRESQL_URL is required for admin sites".into(),
    )
}

fn internal(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
