//! Checkpoint workflow and persist pending approval on human interrupt.

use chrono::{Duration, Utc};

use crate::error::RuntimeError;
use crate::rcs::types::HitlConfig;
use crate::recovery::state::{CompletedStepSnapshot, RecoveryState};
use crate::recovery::storage::RecoveryStorage;
use crate::tracing::emitter::TraceEmitter;
use crate::workflow::{partial_record, RunLoop, WorkflowRunError};
use crate::rcs::types::{ExecutionMode, StepDefinition};

use super::interrupt::HUMAN_INTERRUPT_CODE;
use super::storage::HumanApprovalStorage;

#[allow(clippy::result_large_err)]
pub(crate) fn interrupt_for_human(
    loop_ctx: &RunLoop<'_>,
    legacy: &TraceEmitter,
    _step: &StepDefinition,
    step_index: usize,
    hitl: &HitlConfig,
    recovery_enabled: bool,
) -> Result<(), WorkflowRunError> {
    if !recovery_enabled {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "HITL steps require recovery_enabled".into(),
        }));
    }
    if !hitl.interrupt {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "HITL step requires interrupt=true".into(),
        }));
    }
    if hitl.approval_key.trim().is_empty() {
        return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "HITL approval_key must be non-empty".into(),
        }));
    }

    let expires_at = Utc::now() + Duration::seconds(hitl.timeout_seconds as i64);
    let partial = partial_record(loop_ctx, legacy);

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
    rt.block_on(async {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .map_err(|e| RuntimeError::RecoveryStorageError {
                reason: format!("postgres connect failed: {e}"),
            })?;

        let completed_steps: Vec<CompletedStepSnapshot> = partial
            .step_outputs
            .iter()
            .map(|s| CompletedStepSnapshot {
                step_id: s.step_id.to_string(),
                agent_id: s.agent_id.to_string(),
                content: s.content.clone(),
            })
            .collect();
        let state = RecoveryState {
            recovery_id: uuid::Uuid::new_v4().to_string(),
            original_run_id: loop_ctx.run_id.to_string(),
            workflow_definition_id: loop_ctx.workflow_id.to_string(),
            original_input: loop_ctx.run_input.to_string(),
            completed_steps,
            failed_at_step_index: step_index,
            failure_error_code: HUMAN_INTERRUPT_CODE.to_string(),
            created_at: Utc::now(),
            is_consumed: false,
            execution_mode: ExecutionMode::Linear,
            current_node_id: None,
            graph_iteration_count: 0,
            pending_join: None,
        };
        RecoveryStorage::new(pool.clone())
            .upsert(&state)
            .await?;
        HumanApprovalStorage::new(pool)
            .insert_pending(
                &loop_ctx.run_id.to_string(),
                &hitl.approval_key,
                expires_at,
            )
            .await
    })
    .map_err(WorkflowRunError::Aborted)?;

    Err(WorkflowRunError::Interrupted {
        approval_key: hitl.approval_key.clone(),
        expires_at,
        partial,
    })
}
