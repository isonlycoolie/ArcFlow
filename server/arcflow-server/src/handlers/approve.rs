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

    let exec_config = parse_exec_config(stored.exec_config_json.clone()).map_err(|e| {
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
