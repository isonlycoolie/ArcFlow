//! Minimal workflow bundle types for WASM edge (mirrors RCS JSON shape).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowBundle {
    pub workflow: WorkflowDefinition,
    pub agents: Vec<AgentDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<StepDefinition>,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ExecutionMode {
    #[default]
    Linear,
    Graph,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StepDefinition {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub order: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentDefinition {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub instructions: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunResult {
    pub output: String,
    pub step_count: u32,
    pub status: String,
}

#[derive(Debug)]
pub enum WasmRunError {
    InvalidJson(String),
    UnsupportedMode(String),
    MissingAgent { step_id: Uuid, agent_id: Uuid },
    EmptyWorkflow,
}

impl WasmRunError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidJson(_) => "invalid_json",
            Self::UnsupportedMode(_) => "unsupported_mode",
            Self::MissingAgent { .. } => "missing_agent",
            Self::EmptyWorkflow => "empty_workflow",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidJson(e) => format!("invalid workflow JSON: {e}"),
            Self::UnsupportedMode(mode) => format!("execution mode '{mode}' is not supported on edge"),
            Self::MissingAgent { step_id, agent_id } => {
                format!("agent '{agent_id}' not found for step '{step_id}'")
            }
            Self::EmptyWorkflow => "workflow has no steps".into(),
        }
    }
}

pub fn parse_bundle(json: &str) -> Result<WorkflowBundle, WasmRunError> {
    serde_json::from_str(json).map_err(|e| WasmRunError::InvalidJson(e.to_string()))
}

pub fn parse_input(json: &str) -> Result<String, WasmRunError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| WasmRunError::InvalidJson(e.to_string()))?;
    if let Some(s) = value.as_str() {
        return Ok(s.to_string());
    }
    if let Some(obj) = value.as_object() {
        if let Some(input) = obj.get("input").and_then(|v| v.as_str()) {
            return Ok(input.to_string());
        }
    }
    Ok(json.to_string())
}
