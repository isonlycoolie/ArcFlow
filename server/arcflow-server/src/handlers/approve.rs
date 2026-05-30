//! POST /v1/runs/{run_id}/approve/{approval_key}

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use arcflow_core::human::ApprovalResult;
use arcflow_core::rcs::types::{AgentDefinition, ErrorCode, ExecutionStatus, WorkflowDefinition};
use arcflow_core::workflow::{WorkflowEngine, WorkflowRunError};

use crate::dto::approve::{ApproveRequest, ApproveResponse};
use crate::exec_config::parse_exec_config;
use crate::state::AppState;

pub async fn approve_run(
    State(state): State<Arc<AppState>>,
    Path((run_id, approval_key)): Path<(String, String)>,
    Json(body): Json<ApproveRequest>,
) -> Result<Json<ApproveResponse>, (StatusCode, String)> {
    let store = state
        .runs
        .as_ref()
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "[ArcFlow] ARCFLOW_POSTGRESQL_URL is required".into(),
        ))?;

    let stored = store
        .get(&run_id)
        .await
        .map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, format!("run '{run_id}' not found")))?;

    if stored.status != ExecutionStatus::Interrupted {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("[ArcFlow] run '{run_id}' is not awaiting approval (status={:?})", stored.status),
        ));
    }

    let workflow: WorkflowDefinition = stored
        .workflow_json
        .as_ref()
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "[ArcFlow] run snapshot missing workflow_json".into(),
        ))
        .and_then(|v| {
            serde_json::from_value(v.clone()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("[ArcFlow] invalid workflow snapshot: {e}"),
                )
            })
        })?;

    let agents: Vec<AgentDefinition> = stored
        .agents_json
        .as_ref()
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "[ArcFlow] run snapshot missing agents_json".into(),
        ))
        .and_then(|v| {
            serde_json::from_value(v.clone()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("[ArcFlow] invalid agents snapshot: {e}"),
                )
            })
        })?;

    let agent_map: HashMap<Uuid, AgentDefinition> =
        agents.iter().map(|a| (a.id, a.clone())).collect();

    let exec_config = parse_exec_config(stored.exec_config_json.clone(), None).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("[ArcFlow] invalid exec_config snapshot: {e}"),
        )
    })?;

    let engine = WorkflowEngine::new();
    let approval = ApprovalResult {
        approved: body.approved,
        data: body.data,
    };

    let execution = engine.resume_with_approval(
        &workflow,
        &agent_map,
        &run_id,
        &approval_key,
        approval,
        None,
        None,
        None,
        arcflow_core::providers::default_max_tokens(),
        arcflow_core::providers::default_temperature(),
        &exec_config,
        true,
    );

    let (status, result_json, error_json, message) = match execution {
        Ok(record) => {
            let output = record
                .step_outputs
                .last()
                .map(|s| s.content.clone())
                .unwrap_or_default();
            (
                ExecutionStatus::Completed,
                Some(serde_json::json!({
                    "output": output,
                    "step_count": record.step_outputs.len(),
                })),
                None,
                "Approval recorded, workflow completed".to_string(),
            )
        }
        Err(WorkflowRunError::Failed { error, partial }) => {
            let code = match &error {
                arcflow_core::error::RuntimeError::HumanRejected { .. } => ErrorCode::HumanRejected,
                _ => ErrorCode::StepExecutionFailed,
            };
            (
                ExecutionStatus::Failed,
                None,
                Some(serde_json::json!({
                    "code": format!("{:?}", code),
                    "message": error.to_string(),
                    "step_id": partial.step_outputs.last().map(|s| s.step_id.to_string()),
                })),
                "Approval recorded, workflow failed".to_string(),
            )
        }
        Err(WorkflowRunError::Aborted(err)) => {
            return map_aborted(err);
        }
        Err(WorkflowRunError::Interrupted { .. }) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "[ArcFlow] workflow remained interrupted after approval".into(),
            ));
        }
    };

    store
        .mark_completed(&run_id, status, result_json.clone(), error_json.clone())
        .await
        .map_err(internal)?;

    Ok(Json(ApproveResponse { status, message }))
}

fn internal(err: sqlx::Error) -> (StatusCode, String) {
    tracing::warn!(error = %err, "database error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "[ArcFlow] Database error".into(),
    )
}

fn map_aborted(err: arcflow_core::error::RuntimeError) -> Result<Json<ApproveResponse>, (StatusCode, String)> {
    match err {
        arcflow_core::error::RuntimeError::HumanTimeout { approval_key } => Err((
            StatusCode::GONE,
            format!("[ArcFlow] HumanTimeout: approval '{approval_key}' expired"),
        )),
        arcflow_core::error::RuntimeError::ApprovalNotFound { approval_key } => Err((
            StatusCode::NOT_FOUND,
            format!("[ArcFlow] ApprovalNotFound: '{approval_key}'"),
        )),
        arcflow_core::error::RuntimeError::AlreadyApproved { approval_key } => Err((
            StatusCode::CONFLICT,
            format!("[ArcFlow] AlreadyApproved: '{approval_key}'"),
        )),
        other => Err((StatusCode::BAD_REQUEST, format!("[ArcFlow] {other}"))),
    }
}
