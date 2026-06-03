//! Resume a workflow interrupted for external callback.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::RuntimeError;
use crate::providers::ModelProvider;
use crate::rcs::types::{
    AgentDefinition, ApprovalResult, ExecutionStatus, ExternalOutcomeReport, ExternalOutcomeStatus,
    WorkflowDefinition,
};
use crate::recovery::persist::load_recovery;
use crate::recovery::storage::RecoveryStorage;
use crate::state::ExecutionStepOutput;
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::workflow::{
    run_sorted_steps, ExecutionConfig, ResumeParams, WorkflowExecutionRecord, WorkflowRunError,
};

use super::recovery::{RecoveryAction, RecoveryDecision};

/// Recovery `failure_error_code` for async external bindings.
pub const EXTERNAL_INTERRUPT_CODE: &str = "ExternalCallbackPending";

/// Resumes an externally interrupted workflow after outcome report.
#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
pub fn resume_workflow_with_external_outcome(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    original_run_id: &str,
    binding_id: &str,
    attach_step_id: Uuid,
    report: &ExternalOutcomeReport,
    decision: &RecoveryDecision,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    provider: Option<Arc<dyn ModelProvider>>,
    provider_max_tokens: u32,
    provider_temperature: f32,
    exec_config: &ExecutionConfig,
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    match decision.action {
        RecoveryAction::FailRun => {
            return Err(WorkflowRunError::Failed {
                error: RuntimeError::AgentExecutionFailed {
                    step_id: attach_step_id,
                    reason: report
                        .error_code
                        .clone()
                        .unwrap_or_else(|| "external_binding_failed".into()),
                },
                partial: load_partial(original_run_id)?,
            });
        }
        RecoveryAction::RequestHitl | RecoveryAction::RetryExternal => {
            return Err(WorkflowRunError::Aborted(
                RuntimeError::InvalidWorkflowDefinition {
                    reason: format!(
                        "external recovery action {:?} requires server-side orchestration",
                        decision.action
                    ),
                },
            ));
        }
        RecoveryAction::InjectToolResult | RecoveryAction::ResumeSuccess => {}
    }

    if !exec_config.recovery_enabled {
        return Err(WorkflowRunError::Aborted(
            RuntimeError::InvalidWorkflowDefinition {
                reason: "recovery is not enabled for this run".into(),
            },
        ));
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
            reason: e.to_string(),
        })
    })?;
    let state = rt
        .block_on(load_recovery(original_run_id))
        .map_err(WorkflowRunError::Aborted)?;
    let Some(recovery) = state else {
        return Err(WorkflowRunError::Aborted(
            RuntimeError::RecoveryStorageError {
                reason: format!("no recovery state for run_id '{original_run_id}'"),
            },
        ));
    };
    if recovery.failure_error_code != EXTERNAL_INTERRUPT_CODE {
        return Err(WorkflowRunError::Aborted(
            RuntimeError::InvalidWorkflowDefinition {
                reason: "run is not awaiting external callback".into(),
            },
        ));
    }

    let approved = report.status == ExternalOutcomeStatus::Success;
    let mut data = json!({
        "binding_id": binding_id,
        "status": match report.status {
            ExternalOutcomeStatus::Success => "success",
            ExternalOutcomeStatus::Failed => "failed",
            ExternalOutcomeStatus::NeedsInput => "needs_input",
        },
    });
    if let Some(code) = &report.error_code {
        data["error_code"] = json!(code);
    }
    if let Some(fields) = &report.fields {
        data["fields"] = fields.clone();
    }

    let approval = ApprovalResult { approved, data };

    let precompleted: Vec<ExecutionStepOutput> = recovery
        .completed_steps
        .iter()
        .filter_map(|s| {
            let step_id = Uuid::parse_str(&s.step_id).ok()?;
            let agent_id = Uuid::parse_str(&s.agent_id).ok()?;
            Some(ExecutionStepOutput {
                step_id,
                agent_id,
                content: s.content.clone(),
                status: ExecutionStatus::Completed,
            })
        })
        .collect();

    let original_uuid = Uuid::parse_str(original_run_id).map_err(|_| {
        WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
            reason: "invalid original_run_id".into(),
        })
    })?;

    let resume = ResumeParams {
        original_run_id: original_uuid,
        recovery_id: recovery.recovery_id.clone(),
        precompleted,
        start_step_index: recovery.failed_at_step_index,
        run_input: recovery.original_input.clone(),
        approval: Some(approval),
    };

    let run_input = resume.run_input.clone();
    let record = run_sorted_steps(
        agent_runtime,
        workflow,
        agents,
        &run_input,
        tool_runtime,
        tool_invoker,
        provider,
        provider_max_tokens,
        provider_temperature,
        exec_config,
        Some(resume),
        None,
    )?;

    if let Ok(url) = std::env::var("ARCFLOW_POSTGRESQL_URL") {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
                reason: e.to_string(),
            })
        })?;
        let _ = rt.block_on(async {
            if let Ok(pool) = sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .connect(&url)
                .await
            {
                let storage = RecoveryStorage::new(pool);
                let _ = storage.mark_consumed(&recovery.recovery_id).await;
            }
        });
    }

    Ok(record)
}

fn load_partial(original_run_id: &str) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
            reason: e.to_string(),
        })
    })?;
    let state = rt
        .block_on(load_recovery(original_run_id))
        .map_err(WorkflowRunError::Aborted)?;
    let Some(recovery) = state else {
        return Err(WorkflowRunError::Aborted(
            RuntimeError::RecoveryStorageError {
                reason: format!("no recovery state for run_id '{original_run_id}'"),
            },
        ));
    };
    let run_id = Uuid::parse_str(&recovery.original_run_id).unwrap_or_else(|_| Uuid::new_v4());
    let workflow_id =
        Uuid::parse_str(&recovery.workflow_definition_id).unwrap_or_else(|_| Uuid::new_v4());
    let step_outputs: Vec<ExecutionStepOutput> = recovery
        .completed_steps
        .iter()
        .filter_map(|s| {
            let step_id = Uuid::parse_str(&s.step_id).ok()?;
            let agent_id = Uuid::parse_str(&s.agent_id).ok()?;
            Some(ExecutionStepOutput {
                step_id,
                agent_id,
                content: s.content.clone(),
                status: ExecutionStatus::Completed,
            })
        })
        .collect();
    Ok(WorkflowExecutionRecord {
        run_id,
        workflow_id,
        step_outputs,
        final_state: crate::state::StateEngine::new().snapshot(),
        trace_events: vec![],
    })
}
