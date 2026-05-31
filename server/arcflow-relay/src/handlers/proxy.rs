//! Proxy run requests to arcflow-server.

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use serde_json::Value;

use crate::handlers::policy;
use crate::middleware::site_auth::SiteContext;
use crate::state::RelayState;

pub async fn create_run(
    State(state): State<Arc<RelayState>>,
    Extension(ctx): Extension<SiteContext>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let payload: Value = serde_json::from_slice(&body).map_err(|e| bad_request(&e.to_string()))?;
    policy::enforce_policy(&ctx, &payload)?;

    let url = format!("{}/v1/runs", state.upstream_url);
    let mut req = state
        .http
        .post(&url)
        .header("X-ArcFlow-Api-Key", &ctx.site.upstream_runtime_key)
        .header("Content-Type", "application/json")
        .body(body.to_vec());

    if let Some(key) = headers.get("Idempotency-Key").and_then(|v| v.to_str().ok()) {
        req = req.header("Idempotency-Key", key);
    }

    proxy_json(req).await
}

pub async fn get_run(
    State(state): State<Arc<RelayState>>,
    Extension(ctx): Extension<SiteContext>,
    Path((_site_id, run_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let url = format!("{}/v1/runs/{run_id}", state.upstream_url);
    let req = state
        .http
        .get(&url)
        .header("X-ArcFlow-Api-Key", &ctx.site.upstream_runtime_key);
    proxy_json(req).await
}

pub async fn get_run_trace(
    State(state): State<Arc<RelayState>>,
    Extension(ctx): Extension<SiteContext>,
    Path((_site_id, run_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let url = format!("{}/v1/runs/{run_id}/trace", state.upstream_url);
    let req = state
        .http
        .get(&url)
        .header("X-ArcFlow-Api-Key", &ctx.site.upstream_runtime_key);
    proxy_json(req).await
}

async fn proxy_json(
    req: reqwest::RequestBuilder,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let res = req.send().await.map_err(|e| upstream_error(&e.to_string()))?;
    let status = StatusCode::from_u16(res.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let body = res.text().await.map_err(|e| upstream_error(&e.to_string()))?;
    let json: Value = serde_json::from_str(&body).unwrap_or(Value::String(body));
    Ok((status, Json(json)))
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.into())
}

fn upstream_error(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_GATEWAY, format!("[ArcFlow Relay] Upstream error: {msg}"))
}
