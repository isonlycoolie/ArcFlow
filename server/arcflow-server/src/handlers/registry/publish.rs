//! PUT /v1/workflows/{name}/versions/{version}

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use arcflow_core::rcs::types::AgentDefinition;

use crate::dto::registry::{PublishWorkflowRequest, PublishWorkflowResponse};
use crate::registry::hash;
use crate::state::AppState;

use super::common::{bad_request, internal, registry_store};

pub async fn publish_workflow(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
    Json(body): Json<PublishWorkflowRequest>,
) -> Result<Json<PublishWorkflowResponse>, (StatusCode, String)> {
    semver::Version::parse(&version).map_err(|_| bad_request("version must be valid semver"))?;
    validate_agents(&body.workflow, &body.agents).map_err(bad_request)?;

    let store = registry_store(&state)?;
    let bundle = serde_json::json!({
        "workflow": body.workflow,
        "agents": body.agents,
    });
    let schema_hash = hash::schema_hash(&bundle);
    let published = store
        .publish(
            &name,
            &version,
            &schema_hash,
            bundle,
            body.published_by.as_deref(),
        )
        .await
        .map_err(internal)?;

    Ok(Json(PublishWorkflowResponse {
        name: published.name,
        version: published.version,
        schema_hash: published.schema_hash,
        published_at: published.published_at.to_rfc3339(),
    }))
}

fn validate_agents(
    workflow: &arcflow_core::rcs::types::WorkflowDefinition,
    agents: &[AgentDefinition],
) -> Result<(), String> {
    let ids: HashMap<Uuid, _> = agents.iter().map(|a| (a.id, a)).collect();
    for step in &workflow.steps {
        if !ids.contains_key(&step.agent_id) {
            return Err(format!(
                "agent '{}' not found for step '{}'",
                step.agent_id, step.id
            ));
        }
    }
    Ok(())
}
