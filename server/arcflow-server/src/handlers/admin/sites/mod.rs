//! Admin site create and read handlers.

mod updates;

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::dto::sites::{CreateSiteRequest, CreateSiteResponse, SiteResponse};
use crate::state::AppState;
use crate::store::sites::SiteStore;

pub use updates::{patch_site, rotate_token};

pub async fn create_site(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSiteRequest>,
) -> Result<Json<CreateSiteResponse>, (StatusCode, String)> {
    let store = sites_store(&state)?;
    if body.display_name.trim().is_empty() {
        return Err(bad_request("display_name is required"));
    }
    let upstream = body
        .upstream_runtime_key
        .as_deref()
        .or(state.default_upstream_runtime_key.as_deref())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "[ArcFlow] upstream_runtime_key required (body or ARCFLOW_DEFAULT_UPSTREAM_RUNTIME_KEY)".into(),
        ))?;

    let (site, token) = store
        .create(
            body.display_name.trim(),
            &body.allowed_origins,
            body.rate_limit_rpm.max(1),
            body.allow_inline,
            body.default_workflow_name.trim(),
            upstream,
            body.chat_instructions.as_deref(),
        )
        .await
        .map_err(internal)?;

    let relay_base = relay_public_base();
    Ok(Json(CreateSiteResponse {
        site_id: site.id.clone(),
        relay_url: format!("{relay_base}/v1/sites/{}", site.id),
        site_token: token,
        kb_namespace: site.kb_namespace,
    }))
}

pub async fn get_site(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<SiteResponse>, (StatusCode, String)> {
    let store = sites_store(&state)?;
    let site = store
        .get(&site_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("site not found"))?;
    Ok(Json(to_response(&site)))
}

pub(crate) fn sites_store(state: &AppState) -> Result<&Arc<SiteStore>, (StatusCode, String)> {
    state.sites.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "[ArcFlow] ARCFLOW_POSTGRESQL_URL is required for admin sites".into(),
    ))
}

pub(crate) fn to_response(site: &crate::store::sites::SiteRecord) -> SiteResponse {
    SiteResponse {
        site_id: site.id.clone(),
        display_name: site.display_name.clone(),
        allowed_origins: site.allowed_origins.clone(),
        rate_limit_rpm: site.rate_limit_rpm,
        allow_inline: site.allow_inline,
        default_workflow_name: site.default_workflow_name.clone(),
        kb_namespace: site.kb_namespace.clone(),
        chat_instructions: site.chat_instructions.clone(),
        created_at: site.created_at.to_rfc3339(),
    }
}

pub(crate) fn relay_public_base() -> String {
    std::env::var("ARCFLOW_RELAY_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:8090".into())
        .trim_end_matches('/')
        .to_string()
}

pub(crate) fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.into())
}

pub(crate) fn not_found(msg: &str) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, msg.into())
}

pub(crate) fn internal(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
