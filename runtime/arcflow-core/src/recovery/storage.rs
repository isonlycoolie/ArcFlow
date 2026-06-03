//! PostgreSQL recovery state storage — parameterized queries only.

use sqlx::PgPool;

use crate::constants::RECOVERY_STORAGE_TIMEOUT_SECS;
use crate::error::RuntimeError;
use crate::rcs::types::ExecutionMode;
use crate::recovery::state::RecoveryState;

pub struct RecoveryStorage {
    pool: PgPool,
}

impl RecoveryStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, state: &RecoveryState) -> Result<(), RuntimeError> {
        let state_json =
            serde_json::to_value(state).map_err(|e| RuntimeError::RecoveryStorageError {
                reason: format!("failed to serialize recovery state: {e}"),
            })?;

        tokio::time::timeout(
            std::time::Duration::from_secs(RECOVERY_STORAGE_TIMEOUT_SECS),
            sqlx::query(
                "INSERT INTO arcflow_recovery_state
                 (recovery_id, original_run_id, workflow_def_id, state_json, created_at, is_consumed,
                  execution_mode, current_node_id, graph_iteration_count)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (original_run_id) DO NOTHING",
            )
            .bind(&state.recovery_id)
            .bind(&state.original_run_id)
            .bind(&state.workflow_definition_id)
            .bind(&state_json)
            .bind(state.created_at)
            .bind(state.is_consumed)
            .bind(execution_mode_str(&state.execution_mode))
            .bind(&state.current_node_id)
            .bind(i32::try_from(state.graph_iteration_count).unwrap_or(i32::MAX))
            .execute(&self.pool),
        )
        .await
        .map_err(|_| RuntimeError::RecoveryStorageError {
            reason: format!(
                "recovery state save timed out after {}s",
                RECOVERY_STORAGE_TIMEOUT_SECS
            ),
        })?
        .map_err(|e| RuntimeError::RecoveryStorageError {
            reason: format!("database error saving recovery state: {e}"),
        })?;

        tracing::info!(
            recovery_id = %state.recovery_id,
            original_run_id = %state.original_run_id,
            completed_steps = state.completed_step_count(),
            "recovery state saved"
        );
        Ok(())
    }

    pub async fn upsert(&self, state: &RecoveryState) -> Result<(), RuntimeError> {
        let state_json =
            serde_json::to_value(state).map_err(|e| RuntimeError::RecoveryStorageError {
                reason: format!("failed to serialize recovery state: {e}"),
            })?;

        tokio::time::timeout(
            std::time::Duration::from_secs(RECOVERY_STORAGE_TIMEOUT_SECS),
            sqlx::query(
                "INSERT INTO arcflow_recovery_state
                 (recovery_id, original_run_id, workflow_def_id, state_json, created_at, is_consumed,
                  execution_mode, current_node_id, graph_iteration_count)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (original_run_id) DO UPDATE SET
                   state_json = EXCLUDED.state_json,
                   execution_mode = EXCLUDED.execution_mode,
                   current_node_id = EXCLUDED.current_node_id,
                   graph_iteration_count = EXCLUDED.graph_iteration_count",
            )
            .bind(&state.recovery_id)
            .bind(&state.original_run_id)
            .bind(&state.workflow_definition_id)
            .bind(&state_json)
            .bind(state.created_at)
            .bind(state.is_consumed)
            .bind(execution_mode_str(&state.execution_mode))
            .bind(&state.current_node_id)
            .bind(i32::try_from(state.graph_iteration_count).unwrap_or(i32::MAX))
            .execute(&self.pool),
        )
        .await
        .map_err(|_| RuntimeError::RecoveryStorageError {
            reason: format!(
                "recovery state upsert timed out after {}s",
                RECOVERY_STORAGE_TIMEOUT_SECS
            ),
        })?
        .map_err(|e| RuntimeError::RecoveryStorageError {
            reason: format!("database error upserting recovery state: {e}"),
        })?;

        tracing::info!(
            recovery_id = %state.recovery_id,
            original_run_id = %state.original_run_id,
            node = ?state.current_node_id,
            "graph recovery checkpoint saved"
        );
        Ok(())
    }

    pub async fn load(&self, original_run_id: &str) -> Result<Option<RecoveryState>, RuntimeError> {
        use sqlx::Row;
        let row = tokio::time::timeout(
            std::time::Duration::from_secs(RECOVERY_STORAGE_TIMEOUT_SECS),
            sqlx::query(
                "SELECT state_json FROM arcflow_recovery_state
                 WHERE original_run_id = $1 AND is_consumed = FALSE",
            )
            .bind(original_run_id)
            .fetch_optional(&self.pool),
        )
        .await
        .map_err(|_| RuntimeError::RecoveryStorageError {
            reason: "recovery state load timed out".into(),
        })?
        .map_err(|e| RuntimeError::RecoveryStorageError {
            reason: format!("database error loading recovery state: {e}"),
        })?;

        match row {
            None => Ok(None),
            Some(r) => {
                let state_json: serde_json::Value = r.get("state_json");
                let state: RecoveryState = serde_json::from_value(state_json).map_err(|e| {
                    RuntimeError::RecoveryStorageError {
                        reason: format!("failed to deserialize recovery state: {e}"),
                    }
                })?;
                Ok(Some(state))
            }
        }
    }

    pub async fn mark_consumed(&self, recovery_id: &str) -> Result<(), RuntimeError> {
        tokio::time::timeout(
            std::time::Duration::from_secs(RECOVERY_STORAGE_TIMEOUT_SECS),
            sqlx::query(
                "UPDATE arcflow_recovery_state SET is_consumed = TRUE WHERE recovery_id = $1",
            )
            .bind(recovery_id)
            .execute(&self.pool),
        )
        .await
        .map_err(|_| RuntimeError::RecoveryStorageError {
            reason: "recovery state mark_consumed timed out".into(),
        })?
        .map_err(|e| RuntimeError::RecoveryStorageError {
            reason: format!("database error marking recovery consumed: {e}"),
        })?;
        tracing::info!(recovery_id = %recovery_id, "recovery state marked consumed");
        Ok(())
    }
}

fn execution_mode_str(mode: &ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::Linear => "linear",
        ExecutionMode::Graph => "graph",
    }
}
