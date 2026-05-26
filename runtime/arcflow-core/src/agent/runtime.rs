//! Deterministic stub agent execution (Sprint 2 — no LLM).

use uuid::Uuid;

use crate::rcs::types::{AgentDefinition, ExecutionStatus};
use crate::state::{ExecutionStepOutput, StateSnapshot};

/// Invokes agents without provider I/O; output is derived from role and input.
pub struct AgentRuntime;

impl AgentRuntime {
    /// Builds a stub runtime (stateless).
    pub fn new() -> Self {
        Self
    }

    /// Runs one step: reads `state`, never mutates it.
    pub fn execute(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        state: &StateSnapshot,
        run_input: &str,
    ) -> ExecutionStepOutput {
        let prior = state.steps.len();
        let content = format!(
            "[{role}] processed: {run_input} (step: {step_id}, prior_steps: {prior})",
            role = agent.role,
            run_input = run_input,
            step_id = step_id,
            prior = prior,
        );

        ExecutionStepOutput {
            step_id,
            agent_id: agent.id,
            content,
            status: ExecutionStatus::Completed,
        }
    }
}
