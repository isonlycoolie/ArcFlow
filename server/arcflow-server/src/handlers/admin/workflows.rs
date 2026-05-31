//! Publish default chat workflow for a site.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionMode, MemoryConfig, MemoryScope, MemoryType, StepDefinition,
    WorkflowDefinition,
};

use crate::dto::sites::{PublishChatRequest, PublishChatResponse};
use crate::registry::hash;
use crate::state::AppState;

pub async fn publish_chat(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
    Json(body): Json<PublishChatRequest>,
) -> Result<Json<PublishChatResponse>, (StatusCode, String)> {
    let sites = state.sites.as_ref().ok_or(service_unavailable())?;
    let registry = state.registry.as_ref().ok_or(service_unavailable())?;

    let mut site = sites
        .get(&site_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("site not found"))?;

    if let Some(instructions) = body.instructions.as_deref() {
        site = sites
            .patch(
                &site_id,
                None,
                None,
                None,
                None,
                Some(Some(instructions)),
            )
            .await
            .map_err(internal)?
            .unwrap_or(site);
    }

    let wf_name = site
        .default_workflow_name
        .clone()
        .unwrap_or_else(|| "chat".into());
    let version = body.version.as_deref().unwrap_or("1.0.0");
    semver::Version::parse(version).map_err(|_| bad_request("version must be valid semver"))?;

    let instructions = site
        .chat_instructions
        .clone()
        .unwrap_or_else(|| "You are a helpful assistant.".into());

    let agent_id = Uuid::new_v4();
    let workflow_id = Uuid::new_v4();
    let step_id = Uuid::new_v4();

    let agent = AgentDefinition {
        id: agent_id,
        name: "chat".into(),
        role: "assistant".into(),
        instructions,
        tools: None,
        memory_config: Some(MemoryConfig {
            memory_type: MemoryType::Vector,
            scope: MemoryScope::Agent,
            namespace: Some(site.kb_namespace.clone()),
            ttl_seconds: None,
            embedding: Some("stub/384".into()),
            retrieval: None,
            chunking: None,
        }),
        context: None,
        tool_execution: None,
    };

    let workflow = WorkflowDefinition {
        id: workflow_id,
        name: wf_name.clone(),
        steps: vec![StepDefinition {
            id: step_id,
            agent_id,
            order: 0,
            fallback_step_id: None,
            hitl: None,
        }],
        retry_policy: None,
        execution_mode: ExecutionMode::Linear,
        graph: None,
        external_bindings: None,
    };

    let bundle = serde_json::json!({ "workflow": workflow, "agents": [agent] });
    let schema_hash = hash::schema_hash(&bundle);
    let published = registry
        .publish(
            &wf_name,
            version,
            &schema_hash,
            bundle,
            Some("admin"),
        )
        .await
        .map_err(internal)?;

    Ok(Json(PublishChatResponse {
        name: published.name,
        version: published.version,
        schema_hash: published.schema_hash,
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
        "[ArcFlow] Postgres required for admin workflows".into(),
    )
}

fn internal(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
