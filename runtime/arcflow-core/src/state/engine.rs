//! Append-only execution state for one workflow run.

use uuid::Uuid;

use crate::error::StateError;
use crate::rcs::types::ExecutionStatus;

/// Output of a single stub agent invocation, ready to commit to state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionStepOutput {
    /// Step that produced this output.
    pub step_id: Uuid,
    /// Agent that ran the step.
    pub agent_id: Uuid,
    /// Deterministic stub text (Sprint 2 — no LLM).
    pub content: String,
    /// Outcome for this step.
    pub status: ExecutionStatus,
}

/// Owned, read-only view of committed step outputs for the next agent call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateSnapshot {
    /// Steps committed in execution order.
    pub steps: Vec<ExecutionStepOutput>,
}

/// Holds committed step outputs for one run; grows forward only.
pub struct StateEngine {
    committed_steps: Vec<ExecutionStepOutput>,
}

impl StateEngine {
    /// Starts with no committed steps.
    pub fn new() -> Self {
        Self {
            committed_steps: Vec::new(),
        }
    }

    /// Records a step output, or errors if `step_id` was already committed.
    pub fn commit(&mut self, output: ExecutionStepOutput) -> Result<(), StateError> {
        if self
            .committed_steps
            .iter()
            .any(|s| s.step_id == output.step_id)
        {
            return Err(StateError::DuplicateCommit {
                step_id: output.step_id,
            });
        }
        self.committed_steps.push(output);
        Ok(())
    }

    /// Clones all committed outputs for passing into the agent stub.
    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            steps: self.committed_steps.clone(),
        }
    }

    /// Looks up a committed step by id.
    pub fn step_output(&self, step_id: Uuid) -> Option<&ExecutionStepOutput> {
        self.committed_steps.iter().find(|s| s.step_id == step_id)
    }
}
