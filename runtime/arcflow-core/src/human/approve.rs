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
