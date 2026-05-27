//! Deterministic stub agent execution (Sprint 2 — no LLM).

use serde_json::json;
use uuid::Uuid;

use crate::error::RuntimeError;
use crate::rcs::types::{AgentDefinition, ExecutionStatus};
use crate::state::{ExecutionStepOutput, StateSnapshot};
use crate::tools::ToolError;
use crate::workflow::ExecutionContext;

use super::stub::STUB_FAIL_ROLE;

/// Invokes agents without provider I/O; output is derived from role and input.
pub struct AgentRuntime;

impl Default for AgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRuntime {
    /// Builds a stub runtime (stateless).
    pub fn new() -> Self {
        Self
    }

    /// Runs one step: reads `state`, never mutates it.
    ///
    /// Returns [`RuntimeError::AgentExecutionFailed`] when `agent.role` is [`STUB_FAIL_ROLE`].
    pub fn execute(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        state: &StateSnapshot,
        run_input: &str,
    ) -> Result<ExecutionStepOutput, RuntimeError> {
        self.execute_with_context(agent, step_id, state, run_input, None)
    }

    /// Like [`Self::execute`] with optional tools, memory, and trace context.
    pub fn execute_with_context(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        state: &StateSnapshot,
        run_input: &str,
        ctx: Option<&mut ExecutionContext<'_>>,
    ) -> Result<ExecutionStepOutput, RuntimeError> {
        if let Some(ctx) = ctx {
            self.run_tools_if_configured(agent, step_id, run_input, ctx)?;
        }
        if agent.role == STUB_FAIL_ROLE {
            return Err(RuntimeError::AgentExecutionFailed {
                step_id,
                reason: "stub agent configured to fail".into(),
            });
        }
        let prior = state.steps.len();
        let content = format!(
            "[{role}] processed: {run_input} (step: {step_id}, prior_steps: {prior})",
            role = agent.role,
            run_input = run_input,
            step_id = step_id,
            prior = prior,
        );

        Ok(ExecutionStepOutput {
            step_id,
            agent_id: agent.id,
            content,
            status: ExecutionStatus::Completed,
        })
    }

    fn run_tools_if_configured(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        run_input: &str,
        ctx: &mut ExecutionContext<'_>,
    ) -> Result<(), RuntimeError> {
        let Some(tools) = agent.tools.as_ref() else {
            return Ok(());
        };
        if tools.is_empty() {
            return Ok(());
        }
        let Some(runtime) = ctx.tool_runtime else {
            return Ok(());
        };
        let Some(invoker) = ctx.tool_invoker.clone() else {
            return Ok(());
        };
        let rt = tokio::runtime::Runtime::new().map_err(|e| RuntimeError::ToolExecutionFailed {
            tool_name: "runtime".into(),
            step_id,
            reason: e.to_string(),
        })?;
        for def in tools {
            let input = json!({ "message": run_input });
            if let Err(err) = rt.block_on(runtime.execute_tool(
                &def.name,
                input,
                invoker.clone(),
                ctx.trace,
                Some(step_id),
            )) {
                return Err(map_tool_error(def.name.clone(), step_id, err));
            }
        }
        Ok(())
    }
}

fn map_tool_error(name: String, step_id: Uuid, err: ToolError) -> RuntimeError {
    RuntimeError::ToolExecutionFailed {
        tool_name: name,
        step_id,
        reason: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::agent::STUB_FAIL_ROLE;
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
        let out = rt
            .execute(&agent, sid, &StateSnapshot { steps: vec![] }, "hi")
            .unwrap();
        assert_eq!(out.agent_id, aid);
        assert_eq!(out.status, ExecutionStatus::Completed);
    }

    #[test]
    fn execute_output_step_id_matches_input_step_id() {
        let agent = sample_agent();
        let sid = Uuid::new_v4();
        let out = AgentRuntime::new()
            .execute(&agent, sid, &StateSnapshot { steps: vec![] }, "x")
            .unwrap();
        assert_eq!(out.step_id, sid);
    }

    #[test]
    fn execute_output_includes_agent_role_in_content() {
        let mut agent = sample_agent();
        agent.role = "coder".into();
        let out = AgentRuntime::new()
            .execute(
                &agent,
                Uuid::new_v4(),
                &StateSnapshot { steps: vec![] },
                "task",
            )
            .unwrap();
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
        let out = AgentRuntime::new()
            .execute(&agent, Uuid::new_v4(), &snap, "in")
            .unwrap();
        assert!(out.content.contains("prior_steps: 1"));
    }

    #[test]
    fn execute_stub_fail_role_returns_agent_execution_failed() {
        let mut agent = sample_agent();
        agent.role = STUB_FAIL_ROLE.to_string();
        let sid = Uuid::new_v4();
        let err = AgentRuntime::new()
            .execute(&agent, sid, &StateSnapshot { steps: vec![] }, "in")
            .unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::AgentExecutionFailed { step_id, .. } if step_id == sid
        ));
    }
}
