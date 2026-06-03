//! POST /v1/runs/{run_id}/external/{binding_id}

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use arcflow_core::external::{
    decide_recovery, emit_external_traces, find_binding, validate_outcome_envelope, MonitorContext,
    RecoveryAction,
};
use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionStatus, ExternalOutcomeReport, WorkflowDefinition,
};
use arcflow_core::workflow::{WorkflowEngine, WorkflowRunError};

use crate::dto::external::ExternalCallbackResponse;
use crate::exec_config::parse_exec_config;
use crate::state::AppState;

pub async fn external_callback(
    State(state): State<Arc<AppState>>,
    Path((run_id, binding_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<ExternalCallbackResponse>), (StatusCode, String)> {
    let webhook_secret = state.webhook_secret.as_deref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "[ArcFlow] ARCFLOW_WEBHOOK_SECRET is required for external callbacks".into(),
    ))?;

    let signature = headers
        .get("X-ArcFlow-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "[ArcFlow] missing X-ArcFlow-Signature header".into(),
        ))?;

    if !arcflow_core::external::verify_webhook_signature(webhook_secret, &body, signature) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "[ArcFlow] invalid webhook signature".into(),
        ));
    }

    if let Some(key) = headers
        .get("X-Idempotency-Key")
        .and_then(|v| v.to_str().ok())
    {
        let mut guard = state.external_idempotency.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "[ArcFlow] idempotency store lock failed".into(),
            )
        })?;
        if !guard.insert(format!("{run_id}:{binding_id}:{key}")) {
            return Ok((
                StatusCode::ACCEPTED,
                Json(ExternalCallbackResponse {
                    run_id: run_id.clone(),
                    binding_id: binding_id.clone(),
                    status: "already_processed".into(),
                }),
            ));
        }
    }

    let report: ExternalOutcomeReport = serde_json::from_slice(&body).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("[ArcFlow] invalid ExternalOutcomeReport JSON: {e}"),
        )
    })?;

    let store = state.runs.as_ref().ok_or((
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
            StatusCode::CONFLICT,
            format!(
                "[ArcFlow] run '{run_id}' is not awaiting external callback (status={:?})",
                stored.status
            ),
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

    let binding = find_binding(&workflow, &binding_id).ok_or((
        StatusCode::NOT_FOUND,
        format!("external binding '{binding_id}' not found on workflow"),
    ))?;

    validate_outcome_envelope(binding, &report)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("[ArcFlow] {e}")))?;

    let policy = binding.recovery.clone().unwrap_or_default();
    let decision = decide_recovery(&report, &policy, 0);

    let step_id = binding.attach_to_step_id.to_string();
    emit_external_traces(
        &MonitorContext {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            started_at_ms: 0,
        },
        binding,
        &report,
        &decision,
    );

    if matches!(
        decision.action,
        RecoveryAction::RetryExternal | RecoveryAction::RequestHitl
    ) {
        return Ok((
            StatusCode::ACCEPTED,
            Json(ExternalCallbackResponse {
                run_id: run_id.clone(),
                binding_id: binding_id.clone(),
                status: format!("{:?}", decision.action).to_lowercase(),
            }),
        ));
    }

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
    let execution = engine.resume_with_external_outcome(
        &workflow,
        &agent_map,
        &run_id,
        &binding_id,
        binding.attach_to_step_id,
        &report,
        &decision,
        None,
        None,
        None,
        arcflow_core::providers::default_max_tokens(),
        arcflow_core::providers::default_temperature(),
        &exec_config,
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
                Some(serde_json::json!({ "output": output })),
                None,
                "external callback accepted".into(),
            )
        }
        Err(WorkflowRunError::Failed { partial, .. }) => {
            let output = partial
                .step_outputs
                .last()
                .map(|s| s.content.clone())
                .unwrap_or_default();
            (
                ExecutionStatus::Failed,
                None,
                Some(serde_json::json!({ "partial_output": output })),
                "external binding failed".into(),
            )
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("[ArcFlow] resume failed: {e}"),
            ));
        }
    };

    store
        .mark_completed(&run_id, status, result_json, error_json)
        .await
        .map_err(internal)?;

    Ok((
        StatusCode::ACCEPTED,
        Json(ExternalCallbackResponse {
            run_id,
            binding_id,
            status: message,
        }),
    ))
}

fn internal(err: impl std::fmt::Display) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("[ArcFlow] internal error: {err}"),
    )
}
