//! Resume a human-interrupted workflow after approval.

use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::RuntimeError;
use crate::providers::ModelProvider;
use crate::rcs::types::{AgentDefinition, ApprovalResult, ExecutionStatus, WorkflowDefinition};
use crate::recovery::persist::load_recovery;
use crate::recovery::storage::RecoveryStorage;
use crate::state::ExecutionStepOutput;
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::workflow::{
    run_sorted_steps, ExecutionConfig, ResumeParams, WorkflowExecutionRecord, WorkflowRunError,
};

use super::interrupt::HUMAN_INTERRUPT_CODE;
use super::storage::{ApprovalResolveOutcome, HumanApprovalStorage};

/// Resolves approval in Postgres and resumes the interrupted workflow.
#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
pub fn resume_workflow_with_approval(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    original_run_id: &str,
    approval_key: &str,
    approval: ApprovalResult,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    provider: Option<Arc<dyn ModelProvider>>,
    provider_max_tokens: u32,
    provider_temperature: f32,
    exec_config: &ExecutionConfig,
    resolve_in_db: bool,
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    if !exec_config.recovery_enabled {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "recovery is not enabled for this run".into(),
        }));
    }

    if resolve_in_db {
        let url = std::env::var("ARCFLOW_POSTGRESQL_URL").map_err(|_| {
            WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
                reason: "ARCFLOW_POSTGRESQL_URL is not set".into(),
            })
        })?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
                reason: e.to_string(),
            })
        })?;
        let outcome = rt
            .block_on(async {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(2)
                    .connect(&url)
                    .await
                    .map_err(|e| RuntimeError::RecoveryStorageError {
                        reason: format!("postgres connect failed: {e}"),
                    })?;
                HumanApprovalStorage::new(pool)
                    .resolve(
                        original_run_id,
                        approval_key,
                        approval.approved,
                        &approval.data,
                    )
                    .await
            })
            .map_err(WorkflowRunError::Aborted)?;
        if outcome == ApprovalResolveOutcome::AlreadyResolved && !approval.approved {
            // Idempotent replay of same approval — continue resume below.
        }
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
        return Err(WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
            reason: format!("no recovery state for run_id '{original_run_id}'"),
        }));
    };
    if !recovery.is_resumable() {
        return Err(WorkflowRunError::Aborted(RuntimeError::RecoveryStorageError {
            reason: "recovery state has already been consumed".into(),
        }));
    }
    if recovery.failure_error_code != HUMAN_INTERRUPT_CODE {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "run is not in human interrupt state".into(),
        }));
    }
    if recovery.workflow_definition_id != workflow.id.to_string() {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "workflow definition id does not match recovery state".into(),
        }));
    }

    if !approval.approved {
        return Err(WorkflowRunError::Failed {
            error: RuntimeError::HumanRejected {
                approval_key: approval_key.to_string(),
            },
            partial: build_partial_from_recovery(&recovery),
        });
    }

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

fn build_partial_from_recovery(recovery: &crate::recovery::state::RecoveryState) -> WorkflowExecutionRecord {
    let run_id = Uuid::parse_str(&recovery.original_run_id).unwrap_or_else(|_| Uuid::new_v4());
    let workflow_id =
        Uuid::parse_str(&recovery.workflow_definition_id).unwrap_or_else(|_| Uuid::new_v4());
