//! POST /v1/runs and GET /v1/runs/{run_id}.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use arcflow_core::rcs::types::{AgentDefinition, ErrorCode, ExecutionStatus};
use arcflow_core::tracing::get_execution_trace;
use arcflow_core::workflow::{WorkflowEngine, WorkflowRunError};

use crate::dto::runs::{CreateRunRequest, CreateRunResponse, RunStatusResponse};
use crate::exec_config::parse_exec_config;
use crate::state::AppState;

pub async fn create_run(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateRunRequest>,
) -> Result<Json<CreateRunResponse>, (StatusCode, String)> {
    let store = state
        .runs
        .as_ref()
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "[ArcFlow] ARCFLOW_POSTGRESQL_URL is required for /v1/runs".into(),
        ))?;

    if let Some(key) = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
    {
        if let Some(existing) = store
            .find_by_idempotency(key)
            .await
            .map_err(internal)?
        {
            return Ok(Json(CreateRunResponse {
                run_id: existing.run_id,
                trace_id: existing.trace_id,
                status: existing.status,
            }));
        }
    }

    if body.input.trim().is_empty() {
        return Err(bad_request("input must be non-empty"));
    }

    validate_agents(&body.workflow, &body.agents).map_err(bad_request)?;
    validate_hitl(&body.workflow, body.exec_config.as_ref()).map_err(bad_request)?;

    let run_id = Uuid::new_v4();
    let trace_id = Uuid::new_v4();
    let workflow_hash = body.workflow.id.to_string();
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok());

    store
        .insert_pending(
            &run_id.to_string(),
            &trace_id.to_string(),
            &workflow_hash,
            body.exec_config.clone(),
            idempotency_key,
            Some(serde_json::to_value(&body.workflow).map_err(internal_json)?),
            Some(serde_json::to_value(&body.agents).map_err(internal_json)?),
            Some(body.input.as_str()),
        )
        .await
        .map_err(internal)?;

    store
        .mark_running(&run_id.to_string())
        .await
        .map_err(internal)?;

    let exec_config = parse_exec_config(body.exec_config).map_err(bad_request)?;
    let agent_map: HashMap<Uuid, AgentDefinition> =
        body.agents.iter().map(|a| (a.id, a.clone())).collect();
    let engine = WorkflowEngine::new();
    let execution = engine.execute_with_config(
        &body.workflow,
        &agent_map,
        &body.input,
        None,
        None,
        None,
        arcflow_core::providers::default_max_tokens(),
        arcflow_core::providers::default_temperature(),
        &exec_config,
    );

    let (status, result_json, error_json) = match execution {
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
            )
        }
        Err(WorkflowRunError::Failed { error, partial }) => {
            let code = match &error {
                arcflow_core::error::RuntimeError::AgentExecutionFailed { .. } => {
                    ErrorCode::StepExecutionFailed
                }
                _ => ErrorCode::InternalError,
            };
            (
                ExecutionStatus::Failed,
                None,
                Some(serde_json::json!({
                    "code": format!("{:?}", code),
                    "message": error.to_string(),
                    "step_id": partial.step_outputs.last().map(|s| s.step_id.to_string()),
                })),
            )
        }
        Err(WorkflowRunError::Interrupted {
            approval_key,
            expires_at,
            partial,
        }) => {
            (
                ExecutionStatus::Interrupted,
                Some(serde_json::json!({
                    "approval_key": approval_key,
                    "expires_at": expires_at.to_rfc3339(),
                    "step_index": partial.step_outputs.len(),
                    "run_id": partial.run_id.to_string(),
                })),
                None,
            )
        }
        Err(WorkflowRunError::Aborted(err)) => {
            return Err(bad_request(err.to_string()));
        }
    };

    store
        .mark_completed(
            &run_id.to_string(),
            status,
            result_json.clone(),
            error_json.clone(),
        )
        .await
        .map_err(internal)?;

    Ok(Json(CreateRunResponse {
        run_id: run_id.to_string(),
        trace_id: trace_id.to_string(),
        status,
    }))
}

pub async fn get_run(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<Json<RunStatusResponse>, (StatusCode, String)> {
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
    Ok(Json(stored.into_response()))
}

pub async fn get_run_trace(
    Path(run_id): Path<String>,
) -> Result<Json<arcflow_core::tracing::types::ExecutionTrace>, (StatusCode, String)> {
    match get_execution_trace(&run_id) {
        Some(trace) => Ok(Json(trace)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("[ArcFlow] Trace not found for run '{run_id}'"),
        )),
    }
