//! Deterministic stub agent execution (Sprint 2 — no LLM).

use serde_json::json;
use uuid::Uuid;

use crate::error::RuntimeError;
use crate::memory::MemoryError;
use crate::providers::async_bridge::block_on_provider;
use crate::providers::{
    build_agent_request, default_max_tokens, default_temperature, ProviderCallError,
};
use crate::retry::engine::{execute_with_retry, RetryError};
use crate::rcs::types::{AgentDefinition, ExecutionStatus, MemoryScope, MemoryType};
use crate::state::{ExecutionStepOutput, StateSnapshot};
use crate::streaming::StreamEvent;
use crate::tools::ToolError;
use crate::tracing::events::TraceEventKind;
use crate::tracing::tokens_consumed;
use crate::tracing::TokenUsage;
use crate::workflow::ExecutionContext;

use super::stub::STUB_FAIL_ROLE;

fn effective_provider_timeout(
    ctx: &ExecutionContext<'_, '_>,
) -> Option<std::time::Duration> {
    let mut limit = ctx.step_timeout;
    if let Some(deadline) = ctx.workflow_deadline {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        limit = Some(match limit {
            Some(step_limit) => step_limit.min(remaining),
            None => remaining,
        });
    }
    limit.filter(|d| !d.is_zero())
}

fn is_deadline_elapsed(err: &RuntimeError) -> bool {
    match err {
        RuntimeError::ProviderCallFailed { reason, .. } => {
            reason.contains("deadline") || reason.contains("timed out")
        }
        RuntimeError::StepTimeout { .. } | RuntimeError::WorkflowTimeout { .. } => true,
        _ => false,
    }
}

fn timeout_error_for_context(
    ctx: &ExecutionContext<'_, '_>,
    step_id: &str,
    configured_ms: u64,
    elapsed_ms: u64,
) -> RuntimeError {
    if ctx
        .workflow_deadline
        .is_some_and(|d| std::time::Instant::now() >= d)
    {
        RuntimeError::WorkflowTimeout {
            configured_ms,
            elapsed_ms,
        }
    } else {
        RuntimeError::StepTimeout {
            step_id: step_id.to_string(),
            configured_ms,
            elapsed_ms,
        }
    }
}

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
        mut ctx: Option<&mut ExecutionContext<'_, '_>>,
    ) -> Result<ExecutionStepOutput, RuntimeError> {
        let memory_note = if let Some(ctx) = ctx.as_mut() {
            self.run_memory_if_configured(agent, step_id, state, run_input, ctx)?
        } else {
            None
        };
        if let Some(ctx) = ctx.as_mut() {
            self.run_tools_if_configured(agent, step_id, run_input, ctx)?;
            tokens_consumed(ctx.sprint5, &ctx.run_id, step_id, &agent.name);
        }
        if agent.role == STUB_FAIL_ROLE {
            return Err(RuntimeError::AgentExecutionFailed {
                step_id,
                reason: "stub agent configured to fail".into(),
            });
        }

        if let Some(ctx) = ctx.as_ref() {
            if let Some(ref test) = ctx.test_config {
                let key = crate::workflow::resolve_key(
                    ctx.step_order,
                    &step_id.to_string(),
                    test,
                );
                if let Some(stub_key) = key {
                    if test.should_fail(&stub_key, ctx.test_attempt) {
                        return Err(RuntimeError::AgentExecutionFailed {
                            step_id,
                            reason: format!("test stub failure for {stub_key}"),
                        });
                    }
                    if let Some(output) = test.stub_output(&stub_key, ctx.test_attempt) {
                        return Ok(ExecutionStepOutput {
                            step_id,
                            agent_id: agent.id,
                            content: output,
                            status: ExecutionStatus::Completed,
                        });
                    }
                }
            }
        }

        let step_tokens = if let Some(ctx) = ctx.as_mut() {
            if let Some(provider) = ctx.provider.clone() {
                Some(self.execute_with_provider(agent, step_id, run_input, ctx, provider)?)
            } else {
                None
            }
        } else {
            None
        };

        let prior = state.steps.len();
        let (mut content, _tokens_for_step) = if let Some((text, tokens)) = step_tokens {
            (text, Some(tokens))
        } else {
            (
                format!(
                    "[{role}] processed: {run_input} (step: {step_id}, prior_steps: {prior})",
                    role = agent.role,
                    run_input = run_input,
                    step_id = step_id,
                    prior = prior,
                ),
                None,
            )
        };

        if let Some(note) = memory_note {
            content.push_str(&format!(" memory_read={note:?}"));
        }

        if let Some(ctx) = ctx.as_mut() {
            tokens_consumed(ctx.sprint5, &ctx.run_id, step_id, &agent.name);
        }

        Ok(ExecutionStepOutput {
            step_id,
            agent_id: agent.id,
            content,
            status: ExecutionStatus::Completed,
        })
    }
