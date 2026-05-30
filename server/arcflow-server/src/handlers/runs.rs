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

    let (workflow, agents, registry_version) = if let Some(workflow_ref) = &body.workflow_ref {
        if body.workflow.is_some() || body.agents.is_some() {
            return Err(bad_request(
                "provide workflow_ref or inline workflow and agents, not both",
            ));
        }
        let registry = state.registry.as_ref().ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "[ArcFlow] ARCFLOW_POSTGRESQL_URL is required for workflow_ref".into(),
        ))?;
        let loaded = crate::registry::load_ref::load(
            registry,
            &workflow_ref.name,
            &workflow_ref.version,
        )
        .await
        .map_err(bad_request)?;
        (loaded.workflow, loaded.agents, Some(loaded.version))
    } else {
        let workflow = body
            .workflow
            .ok_or_else(|| bad_request("workflow is required when workflow_ref is omitted"))?;
        let agents = body
            .agents
            .ok_or_else(|| bad_request("agents is required when workflow_ref is omitted"))?;
        (workflow, agents, None)
    };

    validate_agents(&workflow, &agents).map_err(bad_request)?;
    validate_hitl(&workflow, body.exec_config.as_ref()).map_err(bad_request)?;

    let run_id = Uuid::new_v4();
    let trace_id = Uuid::new_v4();
    let workflow_hash = workflow.id.to_string();
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
            Some(serde_json::to_value(&workflow).map_err(internal_json)?),
            Some(serde_json::to_value(&agents).map_err(internal_json)?),
            Some(body.input.as_str()),
        )
        .await
        .map_err(internal)?;

    store
        .mark_running(&run_id.to_string())
        .await
        .map_err(internal)?;

    let mut exec_config = parse_exec_config(body.exec_config, registry_version.as_deref())
        .map_err(bad_request)?;
    exec_config.run_id = Some(run_id);
    let agent_map: HashMap<Uuid, AgentDefinition> =
        agents.iter().map(|a| (a.id, a.clone())).collect();
    let engine = WorkflowEngine::new();
    let execution = engine.execute_with_config(
        &workflow,
        &agent_map,
        &body.input,
        None,
        None,
        None,
        arcflow_core::providers::default_max_tokens(),
        arcflow_core::providers::default_temperature(),
        &exec_config,
        None,
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

    persist_run_trace(&state, &run_id.to_string()).await?;

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
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<Json<arcflow_core::tracing::types::ExecutionTrace>, (StatusCode, String)> {
    if let Some(trace) = get_execution_trace(&run_id) {
        return Ok(Json(trace));
    }
    let Some(traces) = state.traces.as_ref() else {
        return Err((
            StatusCode::NOT_FOUND,
            format!("[ArcFlow] Trace not found for run '{run_id}'"),
        ));
    };
    match traces.load_execution_trace(&run_id).await {
        Ok(Some(trace)) => Ok(Json(trace)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("[ArcFlow] Trace not found for run '{run_id}'"),
        )),
        Err(e) => Err(internal(e)),
    }
}

async fn persist_run_trace(state: &AppState, run_id: &str) -> Result<(), (StatusCode, String)> {
    let Some(traces) = state.traces.as_ref() else {
        return Ok(());
    };
    let Some(trace) = get_execution_trace(run_id) else {
        return Ok(());
    };
    traces
        .persist_events(run_id, &trace.events)
        .await
        .map_err(internal)
}

fn internal(err: sqlx::Error) -> (StatusCode, String) {
    tracing::warn!(error = %err, "database error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "[ArcFlow] Database error".into(),
    )
}

fn bad_request(message: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, format!("[ArcFlow] {}", message.into()))
}

fn internal_json(err: serde_json::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("[ArcFlow] JSON serialization error: {err}"),
    )
}

fn validate_hitl(
    workflow: &arcflow_core::rcs::types::WorkflowDefinition,
    exec_config: Option<&serde_json::Value>,
) -> Result<(), String> {
    let has_hitl = workflow.steps.iter().any(|s| s.hitl.is_some());
    if !has_hitl {
        return Ok(());
    }
    let recovery = exec_config
        .and_then(|v| v.get("recovery_enabled"))
        .and_then(|b| b.as_bool())
        .unwrap_or(false);
    if !recovery {
        return Err(
            "workflows with HITL steps require exec_config.recovery_enabled=true".into(),
        );
    }
    Ok(())
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
