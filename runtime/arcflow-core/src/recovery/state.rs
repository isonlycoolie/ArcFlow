//! Recovery state — never log `original_input` or step output values in traces/logs.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rcs::types::ExecutionMode;

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
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_node_id: Option<String>,
    #[serde(default)]
    pub graph_iteration_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_join: Option<HashMap<String, Vec<String>>>,
}

impl RecoveryState {
    pub fn completed_step_count(&self) -> usize {
        self.completed_steps.len()
    }

    pub fn is_resumable(&self) -> bool {
        !self.is_consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_state_graph_fields_default_linear() {
        let state = RecoveryState {
            recovery_id: "r1".into(),
            original_run_id: "run1".into(),
            workflow_definition_id: "wf1".into(),
            original_input: "in".into(),
            completed_steps: vec![],
            failed_at_step_index: 0,
            failure_error_code: "err".into(),
            created_at: Utc::now(),
            is_consumed: false,
            execution_mode: ExecutionMode::Linear,
            current_node_id: None,
            graph_iteration_count: 0,
            pending_join: None,
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let back: RecoveryState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.execution_mode, ExecutionMode::Linear);
        assert!(back.current_node_id.is_none());
    }
}
