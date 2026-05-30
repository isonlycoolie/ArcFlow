//! REST DTOs for `/v1/runs`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use arcflow_core::rcs::types::{AgentDefinition, ExecutionStatus, WorkflowDefinition};

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub workflow: WorkflowDefinition,
    pub agents: Vec<AgentDefinition>,
    pub input: String,
    #[serde(default)]
    pub exec_config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub run_id: String,
    pub trace_id: String,
    pub status: ExecutionStatus,
}

#[derive(Debug, Serialize)]
pub struct RunStatusResponse {
    pub run_id: String,
    pub trace_id: String,
    pub status: ExecutionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<RunResultDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RunErrorDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt: Option<RunInterruptDto>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunInterruptDto {
    pub approval_key: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_index: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct RunResultDto {
    pub output: String,
    pub step_count: usize,
}

#[derive(Debug, Serialize)]
pub struct RunErrorDto {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct StoredRun {
    pub run_id: String,
    pub trace_id: String,
    pub status: ExecutionStatus,
    pub result_json: Option<serde_json::Value>,
    pub error_json: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub workflow_json: Option<serde_json::Value>,
    pub agents_json: Option<serde_json::Value>,
    pub input_text: Option<String>,
    pub exec_config_json: Option<serde_json::Value>,
}

impl StoredRun {
    pub fn into_response(self) -> RunStatusResponse {
        let result = self.result_json.as_ref().and_then(|v| {
            Some(RunResultDto {
                output: v.get("output")?.as_str()?.to_string(),
                step_count: v.get("step_count")?.as_u64()? as usize,
            })
        });
        let error = self.error_json.as_ref().and_then(|v| {
            Some(RunErrorDto {
                code: v.get("code")?.as_str()?.to_string(),
                message: v.get("message")?.as_str()?.to_string(),
                step_id: v
                    .get("step_id")
                    .and_then(|s| s.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok()),
            })
        });
        let interrupt = if self.status == ExecutionStatus::Interrupted {
            self.result_json.as_ref().and_then(|v| {
                Some(RunInterruptDto {
                    approval_key: v.get("approval_key")?.as_str()?.to_string(),
                    expires_at: v.get("expires_at")?.as_str()?.to_string(),
                    step_index: v.get("step_index").and_then(|s| s.as_u64()).map(|n| n as usize),
                })
            })
        } else {
            None
        };
        RunStatusResponse {
            run_id: self.run_id,
            trace_id: self.trace_id,
            status: self.status,
            result,
            error,
            interrupt,
            created_at: self.created_at.to_rfc3339(),
            completed_at: self.completed_at.map(|t| t.to_rfc3339()),
        }
    }
}
