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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::rcs::types::AgentDefinition;
    use crate::state::StateEngine;

    fn sample_agent() -> AgentDefinition {
        AgentDefinition {
            id: Uuid::new_v4(),
            name: "n".into(),
            role: "researcher".into(),
            instructions: "i".into(),
            tools: None,
            memory_config: None,
        }
    }

    #[test]
    fn execute_returns_success_output_with_correct_agent_id() {
        let agent = sample_agent();
        let aid = agent.id;
        let sid = Uuid::new_v4();
        let rt = AgentRuntime::new();
        let out = rt.execute(&agent, sid, &StateSnapshot { steps: vec![] }, "hi");
        assert_eq!(out.agent_id, aid);
        assert_eq!(out.status, ExecutionStatus::Completed);
    }

    #[test]
    fn execute_output_step_id_matches_input_step_id() {
        let agent = sample_agent();
        let sid = Uuid::new_v4();
        let out = AgentRuntime::new().execute(&agent, sid, &StateSnapshot { steps: vec![] }, "x");
        assert_eq!(out.step_id, sid);
    }

    #[test]
    fn execute_output_includes_agent_role_in_content() {
        let mut agent = sample_agent();
        agent.role = "coder".into();
        let out = AgentRuntime::new().execute(
            &agent,
            Uuid::new_v4(),
            &StateSnapshot { steps: vec![] },
            "task",
        );
        assert!(out.content.contains("[coder]"));
    }

    #[test]
    fn execute_output_content_reflects_prior_step_count() {
        let agent = sample_agent();
        let mut st = StateEngine::new();
        let s1 = Uuid::new_v4();
        st.commit(ExecutionStepOutput {
            step_id: s1,
            agent_id: agent.id,
            content: "a".into(),
            status: ExecutionStatus::Completed,
        })
        .unwrap();
        let snap = st.snapshot();
        let out = AgentRuntime::new().execute(&agent, Uuid::new_v4(), &snap, "in");
        assert!(out.content.contains("prior_steps: 1"));
    }
}
