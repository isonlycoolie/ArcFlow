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
