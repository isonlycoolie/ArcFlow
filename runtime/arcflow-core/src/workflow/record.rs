//! Result of a completed or partial workflow run.

use uuid::Uuid;

use crate::state::{ExecutionStepOutput, StateSnapshot};

/// Record returned when a workflow run finishes or halts mid-run.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowExecutionRecord {
    /// Unique id for this execution attempt.
    pub run_id: Uuid,
    /// Workflow definition that was executed.
    pub workflow_id: Uuid,
    /// Outputs for steps that completed before halt or end of run.
    pub step_outputs: Vec<ExecutionStepOutput>,
    /// Committed state after the last successful step.
    pub final_state: StateSnapshot,
}
