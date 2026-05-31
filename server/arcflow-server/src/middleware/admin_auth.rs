//! Admin API key authentication.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use subtle::ConstantTimeEq;

use crate::state::AppState;

pub async fn require_admin_key(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let Some(expected) = state.admin_api_key.as_deref() else {
        return Err(service_unavailable());
    };
    let provided = req
        .headers()
        .get("X-ArcFlow-Admin-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided.is_empty() || !bool::from(provided.as_bytes().ct_eq(expected.as_bytes())) {
        return Err(unauthorized());
    }
    Ok(next.run(req).await)
}

fn unauthorized() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": { "code": "admin_auth_failed", "message": "[ArcFlow] Invalid admin API key." }
        })),
    )
}

fn service_unavailable() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": { "code": "admin_disabled", "message": "[ArcFlow] ARCFLOW_ADMIN_API_KEY is not configured." }
        })),
    )
}
