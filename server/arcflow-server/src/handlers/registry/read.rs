//! GET resolve and alias endpoints for workflow registry.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::dto::registry::{ResolveQuery, SetAliasRequest, WorkflowDefinitionResponse};
use crate::registry::resolve;
use crate::state::AppState;

use super::common::{bad_request, internal, not_found, registry_store, to_definition_response};

pub async fn get_workflow_version(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<WorkflowDefinitionResponse>, (StatusCode, String)> {
    let store = registry_store(&state)?;
    let published = store
        .get(&name, &version)
        .await
        .map_err(internal)?
        .ok_or(not_found(&format!("workflow '{name}' version '{version}'")))?;
    Ok(Json(to_definition_response(published)))
}

pub async fn resolve_workflow(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<ResolveQuery>,
) -> Result<Json<WorkflowDefinitionResponse>, (StatusCode, String)> {
    let store = registry_store(&state)?;
    let versions = store.list_versions(&name).await.map_err(internal)?;
    let version = resolve::pick_matching_version(&versions, &query.range).ok_or(not_found(
        &format!("no workflow '{name}' version matching '{}'", query.range),
    ))?;
    let published = store
        .get(&name, &version)
        .await
        .map_err(internal)?
        .ok_or(not_found(&format!("workflow '{name}' version '{version}'")))?;
    Ok(Json(to_definition_response(published)))
}

pub async fn set_workflow_alias(
    State(state): State<Arc<AppState>>,
    Path((name, alias)): Path<(String, String)>,
    Json(body): Json<SetAliasRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    semver::Version::parse(&body.version).map_err(|_| bad_request("version must be valid semver"))?;
    let store = registry_store(&state)?;
    store
        .get(&name, &body.version)
        .await
        .map_err(internal)?
        .ok_or(not_found(&format!(
            "workflow '{name}' version '{}' not found",
            body.version
        )))?;
    store
        .set_alias(&name, &alias, &body.version)
        .await
        .map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}
