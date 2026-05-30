//! API key authentication (constant-time compare).

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use subtle::ConstantTimeEq;

use crate::state::AppState;

pub const API_KEY_HEADER: &str = "X-ArcFlow-Api-Key";

pub async fn require_api_key(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let provided = req
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ok = provided
        .as_bytes()
        .ct_eq(state.api_key.as_bytes())
        .into();
    if ok {
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": {
                    "code": "authentication_failed",
                    "message": "[ArcFlow] Authentication failed. Provide X-ArcFlow-Api-Key."
                }
            })),
        ))
    }
}
