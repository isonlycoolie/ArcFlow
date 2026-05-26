use uuid::Uuid;

use crate::state::{ExecutionStepOutput, StateSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowExecutionRecord {
    pub run_id: Uuid,
    pub workflow_id: Uuid,
    pub step_outputs: Vec<ExecutionStepOutput>,
    pub final_state: StateSnapshot,
}
