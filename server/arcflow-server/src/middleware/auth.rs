//! API key authentication (constant-time compare) with static runtime keys.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use subtle::ConstantTimeEq;

use crate::state::AppState;

use super::principal::{AuthPrincipal, StaticKeyMap};

pub const API_KEY_HEADER: &str = "X-ArcFlow-Api-Key";
#[allow(dead_code)]
pub const AUTH_PRINCIPAL_EXTENSION: &str = "arcflow.auth.principal";

pub async fn require_api_key(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let provided = extract_api_key(req.headers());
    let principal = resolve_principal(provided, &state.api_key, &state.static_runtime_keys);
    let Some(principal) = principal else {
        return Err(unauthorized());
    };
    req.extensions_mut().insert(principal);
    Ok(next.run(req).await)
}

fn extract_api_key(headers: &axum::http::HeaderMap) -> &str {
    if let Some(v) = headers.get(API_KEY_HEADER).and_then(|v| v.to_str().ok()) {
        if !v.is_empty() {
            return v;
        }
    }
    if let Some(v) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = v.strip_prefix("Bearer ") {
            if !token.is_empty() {
                return token;
            }
        }
    }
    ""
}

fn resolve_principal(
    provided: &str,
    master: &str,
    static_keys: &StaticKeyMap,
) -> Option<AuthPrincipal> {
    if provided.is_empty() {
        return None;
    }
    if provided.as_bytes().ct_eq(master.as_bytes()).into() {
        return Some(AuthPrincipal::Master);
    }
    static_keys
        .get(provided)
        .cloned()
        .map(AuthPrincipal::Runtime)
}

fn unauthorized() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": {
                "code": "authentication_failed",
                "message": "[ArcFlow] Authentication failed. Provide X-ArcFlow-Api-Key or Authorization: Bearer."
            }
        })),
    )
}
