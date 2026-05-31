//! Site token + Origin validation.

use std::sync::Arc;

use arcflow_server::store::sites::SiteStore;
use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::state::RelayState;

#[derive(Clone)]
pub struct SiteContext {
    pub site: arcflow_server::store::sites::SiteRecord,
}

pub async fn require_site_auth(
    State(state): State<Arc<RelayState>>,
    Path(site_id): Path<String>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let token = extract_bearer(req.headers());
    if token.is_empty() {
        return Err(unauthorized());
    }

    let Some(site) = state.resolve_site(&site_id, token).await else {
        return Err(unauthorized());
    };

    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !SiteStore::origin_allowed(&site, origin) {
        return Err(forbidden("origin not allowed"));
    }

    if !state.check_rate_limit(&site) {
        return Err(too_many_requests());
    }

    req.extensions_mut().insert(SiteContext { site });
    Ok(next.run(req).await)
}

fn extract_bearer(headers: &axum::http::HeaderMap) -> &str {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("")
}

fn unauthorized() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": { "code": "site_auth_failed", "message": "[ArcFlow Relay] Invalid site token." }
        })),
    )
}

fn forbidden(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": { "code": "forbidden", "message": msg }
        })),
    )
}

fn too_many_requests() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({
            "error": { "code": "rate_limited", "message": "[ArcFlow Relay] Rate limit exceeded." }
        })),
    )
}
