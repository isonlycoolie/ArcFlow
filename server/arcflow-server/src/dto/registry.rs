//! REST DTOs for workflow registry endpoints.

use serde::{Deserialize, Serialize};

use arcflow_core::rcs::types::{AgentDefinition, WorkflowDefinition};

#[derive(Debug, Deserialize)]
pub struct PublishWorkflowRequest {
    pub workflow: WorkflowDefinition,
    pub agents: Vec<AgentDefinition>,
    #[serde(default)]
    pub published_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishWorkflowResponse {
    pub name: String,
    pub version: String,
    pub schema_hash: String,
    pub published_at: String,
}

#[derive(Debug, Serialize)]
pub struct WorkflowDefinitionResponse {
    pub name: String,
    pub version: String,
    pub schema_hash: String,
    pub definition: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SetAliasRequest {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ResolveQuery {
    pub range: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowRef {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowBundle {
    pub workflow: WorkflowDefinition,
    pub agents: Vec<AgentDefinition>,
}
