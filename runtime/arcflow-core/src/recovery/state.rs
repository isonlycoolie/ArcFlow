//! Recovery state — never log `original_input` or step output values in traces/logs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedStepSnapshot {
    pub step_id: String,
    pub agent_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryState {
    pub recovery_id: String,
    pub original_run_id: String,
    pub workflow_definition_id: String,
    pub original_input: String,
    pub completed_steps: Vec<CompletedStepSnapshot>,
    pub failed_at_step_index: usize,
    pub failure_error_code: String,
    pub created_at: DateTime<Utc>,
    pub is_consumed: bool,
}

impl RecoveryState {
    pub fn completed_step_count(&self) -> usize {
        self.completed_steps.len()
    }

    pub fn is_resumable(&self) -> bool {
        !self.is_consumed
    }
}
