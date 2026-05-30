//! POST /v1/workflows/run — RCS envelope handler (Sprint 8).

use std::sync::Arc;

use axum::{extract::State, Json};
use serde_json::Value;
use arcflow_core::rcs::envelope::MessageEnvelope;
use arcflow_core::rcs::types::{
    ErrorCode, ErrorPayload, ExecutionStatus, MessageType, RunResult,
};
use arcflow_core::workflow::WorkflowEngine;

use crate::state::AppState;

pub async fn run_workflow(
    State(_state): State<Arc<AppState>>,
    Json(envelope): Json<MessageEnvelope>,
) -> Json<MessageEnvelope> {
    let trace_id = envelope.trace_id;
    tracing::info!(%trace_id, "workflow run request received");

    let _engine = WorkflowEngine::new();
    let result = RunResult {
        trace_id,
        status: ExecutionStatus::Failed,
        output: None,
        steps: vec![],
        error: Some(ErrorPayload {
            code: ErrorCode::InternalError,
            message: "[ArcFlow] Full remote workflow execution wiring is pending.".into(),
            step_id: None,
            recoverable: false,
        }),
    };

    Json(MessageEnvelope {
        rcs_version: envelope.rcs_version,
        message_type: MessageType::WorkflowResult,
        trace_id,
        timestamp: chrono::Utc::now(),
        payload: serde_json::to_value(&result).unwrap_or(Value::Null),
    })
}
