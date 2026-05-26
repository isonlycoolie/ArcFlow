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

impl Default for StateEngine {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_output(step_id: Uuid) -> ExecutionStepOutput {
        ExecutionStepOutput {
            step_id,
            agent_id: Uuid::new_v4(),
            content: "x".into(),
            status: ExecutionStatus::Completed,
        }
    }

    #[test]
    fn new_state_engine_has_no_committed_steps() {
        let engine = StateEngine::new();
        assert!(engine.snapshot().steps.is_empty());
    }

    #[test]
    fn commit_adds_step_to_state() {
        let mut engine = StateEngine::new();
        let sid = Uuid::new_v4();
        engine.commit(sample_output(sid)).unwrap();
        assert_eq!(engine.snapshot().steps.len(), 1);
    }

    #[test]
    fn snapshot_reflects_all_committed_steps() {
        let mut engine = StateEngine::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        engine.commit(sample_output(a)).unwrap();
        engine.commit(sample_output(b)).unwrap();
        let snap = engine.snapshot();
        assert_eq!(snap.steps.len(), 2);
    }

    #[test]
    fn commit_returns_error_on_duplicate_step_id() {
        let mut engine = StateEngine::new();
        let sid = Uuid::new_v4();
        engine.commit(sample_output(sid)).unwrap();
        let err = engine.commit(sample_output(sid)).unwrap_err();
        assert_eq!(err, StateError::DuplicateCommit { step_id: sid });
    }

    #[test]
    fn step_output_returns_none_for_unknown_step() {
        let engine = StateEngine::new();
        assert!(engine.step_output(Uuid::new_v4()).is_none());
    }

    #[test]
    fn step_output_returns_correct_output_for_known_step() {
        let mut engine = StateEngine::new();
        let sid = Uuid::new_v4();
        let out = sample_output(sid);
        engine.commit(out.clone()).unwrap();
        assert_eq!(engine.step_output(sid), Some(&out));
    }

    #[test]
    fn committed_state_is_not_mutable_after_commit() {
        let mut engine = StateEngine::new();
        let sid = Uuid::new_v4();
        engine.commit(sample_output(sid)).unwrap();
        assert_eq!(engine.snapshot().steps.len(), 1);

        let mut snap = engine.snapshot();
        snap.steps.clear();
        assert!(snap.steps.is_empty());
        assert_eq!(engine.snapshot().steps.len(), 1);
    }
}
