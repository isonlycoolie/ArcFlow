//! Best-effort recovery persistence on workflow failure.

use chrono::Utc;
use uuid::Uuid;

use crate::recovery::state::{CompletedStepSnapshot, RecoveryState};
use crate::recovery::storage::RecoveryStorage;
use crate::state::ExecutionStepOutput;

pub fn persist_if_enabled(
    enabled: bool,
    workflow_definition_id: Uuid,
    original_run_id: Uuid,
    original_input: &str,
    completed: &[ExecutionStepOutput],
    failed_at_step_index: usize,
    failure_error_code: &str,
) {
    if !enabled {
        return;
    }
    let Ok(url) = std::env::var("ARCFLOW_POSTGRESQL_URL") else {
        tracing::warn!("recovery enabled but ARCFLOW_POSTGRESQL_URL is unset");
        return;
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "failed to create runtime for recovery save");
            return;
        }
    };
    let _ = rt.block_on(async {
        let pool = match sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, "recovery postgres connect failed");
                return;
            }
        };
        let completed_steps: Vec<CompletedStepSnapshot> = completed
            .iter()
            .map(|s| CompletedStepSnapshot {
                step_id: s.step_id.to_string(),
                agent_id: s.agent_id.to_string(),
                content: s.content.clone(),
            })
            .collect();
        let state = RecoveryState {
            recovery_id: Uuid::new_v4().to_string(),
            original_run_id: original_run_id.to_string(),
            workflow_definition_id: workflow_definition_id.to_string(),
            original_input: original_input.to_string(),
            completed_steps,
            failed_at_step_index,
            failure_error_code: failure_error_code.to_string(),
            created_at: Utc::now(),
            is_consumed: false,
            execution_mode: crate::rcs::types::ExecutionMode::Linear,
            current_node_id: None,
            graph_iteration_count: 0,
            pending_join: None,
        };
        let storage = RecoveryStorage::new(pool);
        if let Err(e) = storage.save(&state).await {
            tracing::warn!(error = %e, "recovery save failed");
        }
    });
}

pub async fn load_recovery(
    original_run_id: &str,
) -> Result<Option<RecoveryState>, crate::error::RuntimeError> {
    let url = std::env::var("ARCFLOW_POSTGRESQL_URL").map_err(|_| {
        crate::error::RuntimeError::RecoveryStorageError {
            reason: "ARCFLOW_POSTGRESQL_URL is not set".into(),
        }
    })?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .map_err(|e| crate::error::RuntimeError::RecoveryStorageError {
            reason: format!("postgres connect failed: {e}"),
        })?;
    RecoveryStorage::new(pool).load(original_run_id).await
}
