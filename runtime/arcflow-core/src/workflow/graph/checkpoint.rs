//! Graph recovery checkpoint persistence (Phase 1.1).

use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::rcs::types::ExecutionMode;
use crate::recovery::state::{CompletedStepSnapshot, RecoveryState};
use crate::recovery::storage::RecoveryStorage;
use crate::state::ExecutionStepOutput;

/// Best-effort upsert of graph execution progress when recovery is enabled.
pub fn persist_graph_checkpoint(
    enabled: bool,
    workflow_definition_id: Uuid,
    original_run_id: Uuid,
    original_input: &str,
    completed: &[ExecutionStepOutput],
    current_node_id: &str,
    graph_iteration_count: u32,
    pending_join: Option<HashMap<String, Vec<String>>>,
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
            tracing::warn!(error = %e, "failed to create runtime for graph checkpoint");
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
            failed_at_step_index: completed.len().saturating_sub(1),
            failure_error_code: "graph_checkpoint".into(),
            created_at: Utc::now(),
            is_consumed: false,
            execution_mode: ExecutionMode::Graph,
            current_node_id: Some(current_node_id.to_string()),
            graph_iteration_count,
            pending_join,
        };
        let storage = RecoveryStorage::new(pool);
        if let Err(e) = storage.upsert(&state).await {
            tracing::warn!(error = %e, "graph checkpoint save failed");
        }
    });
}
