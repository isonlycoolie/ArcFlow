//! Admin site patch and token rotation.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::dto::sites::{PatchSiteRequest, RotateTokenResponse, SiteResponse};
use crate::state::AppState;

use super::{internal, not_found, to_response};

pub async fn patch_site(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
    Json(body): Json<PatchSiteRequest>,
) -> Result<Json<SiteResponse>, (StatusCode, String)> {
    let store = super::sites_store(&state)?;
    let instructions = body.chat_instructions.map(Some);
    let site = store
        .patch(
            &site_id,
            body.display_name.as_deref(),
            body.allowed_origins.as_deref(),
            body.rate_limit_rpm,
            body.allow_inline,
            instructions.as_ref().map(|v| v.as_deref()),
        )
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("site not found"))?;
    Ok(Json(to_response(&site)))
}

pub async fn rotate_token(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<RotateTokenResponse>, (StatusCode, String)> {
    let store = super::sites_store(&state)?;
    let token = store
        .rotate_token(&site_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("site not found"))?;
    Ok(Json(RotateTokenResponse { site_token: token }))
}
