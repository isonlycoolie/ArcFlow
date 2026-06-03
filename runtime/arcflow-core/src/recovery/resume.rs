//! Resume a workflow from persisted recovery state.

use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::RuntimeError;
use crate::providers::ModelProvider;
use crate::rcs::types::{AgentDefinition, ExecutionStatus, WorkflowDefinition};
use crate::recovery::persist::load_recovery;
use crate::recovery::storage::RecoveryStorage;
use crate::state::ExecutionStepOutput;
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::workflow::{
    run_sorted_steps, ExecutionConfig, ResumeParams, WorkflowExecutionRecord, WorkflowRunError,
};

/// Loads recovery for `original_run_id`, replays completed steps, continues from failure index.
#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
pub fn resume_workflow(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    original_run_id: &str,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    provider: Option<Arc<dyn ModelProvider>>,
    provider_max_tokens: u32,
    provider_temperature: f32,
    exec_config: &ExecutionConfig,
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    if !exec_config.recovery_enabled {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "recovery is not enabled for this run".into(),
        }));
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
    };
    if recovery.workflow_definition_id != workflow.id.to_string() {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "workflow definition id does not match recovery state".into(),
        }));
    }

    #[cfg(feature = "otel")]
    crate::tracing::otel_metrics::record_recovery_resume();

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
        start_step_index: if recovery.failure_error_code == crate::human::HUMAN_INTERRUPT_CODE {
            recovery.failed_at_step_index
        } else {
            recovery.failed_at_step_index.saturating_add(1)
        },
        run_input: recovery.original_input.clone(),
        approval: None,
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
        rt.block_on(async {
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
