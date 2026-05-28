//! Deterministic stub agent execution (Sprint 2 — no LLM).

use serde_json::json;
use uuid::Uuid;

use crate::error::RuntimeError;
use crate::memory::MemoryError;
use crate::providers::async_bridge::block_on_provider;
use crate::providers::{
    build_agent_request, default_max_tokens, default_temperature, ProviderCallError,
};
use crate::rcs::types::{AgentDefinition, ExecutionStatus, MemoryScope, MemoryType};
use crate::state::{ExecutionStepOutput, StateSnapshot};
use crate::tools::ToolError;
use crate::tracing::events::TraceEventKind;
use crate::tracing::tokens_consumed;
use crate::tracing::TokenUsage;
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

    fn execute_with_provider(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        run_input: &str,
        ctx: &mut ExecutionContext<'_, '_>,
        provider: std::sync::Arc<dyn crate::providers::ModelProvider>,
    ) -> Result<(String, TokenUsage), RuntimeError> {
        let max_tokens = if ctx.provider.is_some() {
            ctx.provider_max_tokens
        } else {
            default_max_tokens()
        };
        let temperature = if ctx.provider.is_some() {
            ctx.provider_temperature
        } else {
            default_temperature()
        };
        let request = build_agent_request(&agent.instructions, run_input, max_tokens, temperature);
        let prompt_size = request.prompt_size_bytes();
        let step_id_str = step_id.to_string();

        ctx.sprint5.emit(TraceEventKind::ProviderRequestSent {
            run_id: ctx.run_id.clone(),
            step_id: step_id_str.clone(),
            provider_id: provider.provider_id().to_string(),
            model_id: provider.model_id().to_string(),
            max_tokens: request.max_tokens,
            prompt_size_bytes: prompt_size,
        });

        let started = std::time::Instant::now();
        let result = block_on_provider(provider.complete(request));

        match result {
            Ok(response) => {
                let latency_ms = started.elapsed().as_millis() as u64;
                ctx.sprint5.emit(TraceEventKind::ProviderResponseReceived {
                    run_id: ctx.run_id.clone(),
                    step_id: step_id_str.clone(),
                    provider_id: provider.provider_id().to_string(),
                    model_id: response.model_id.clone(),
                    tokens: response.tokens.clone(),
                    latency_ms,
                });
                ctx.sprint5.emit(TraceEventKind::AgentResponseReceived {
                    run_id: ctx.run_id.clone(),
                    step_id: step_id_str.clone(),
                    agent_name: agent.name.clone(),
                    output_size_bytes: response.content_size_bytes(),
                });
                ctx.sprint5.emit(TraceEventKind::TokensConsumed {
                    run_id: ctx.run_id.clone(),
                    step_id: step_id_str,
                    agent_name: agent.name.clone(),
                    tokens: response.tokens.clone(),
                });
                Ok((response.content, response.tokens))
            }
            Err(err) => {
                emit_provider_error(ctx, &step_id_str, &err);
                Err(map_provider_error(step_id, err))
            }
        }
    }

    fn run_tools_if_configured(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        run_input: &str,
        ctx: &mut ExecutionContext<'_, '_>,
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
                ctx.legacy,
                ctx.sprint5,
                &ctx.run_id,
                Some(step_id),
            )) {
                return Err(map_tool_error(def.name.clone(), step_id, err));
            }
        }
        Ok(())
    }

    /// Reads prior stub context, writes current input, returns prior value for output annotation.
    fn run_memory_if_configured(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        state: &StateSnapshot,
        run_input: &str,
        ctx: &mut ExecutionContext<'_, '_>,
    ) -> Result<Option<String>, RuntimeError> {
        let Some(config) = agent.memory_config.as_ref() else {
            return Ok(None);
        };
        let value = run_input.as_bytes();
        let prior = match config.memory_type {
            MemoryType::Session => {
                let prior = ctx
                    .memory
                    .read_session(
                        agent.id,
                        STUB_MEMORY_KEY,
                        &agent.name,
                        ctx.legacy,
                        ctx.sprint5,
                        &ctx.run_id,
                        Some(step_id),
                    )
                    .map_err(|e| map_memory_error(step_id, e))?;
                ctx.memory
                    .write_session(
                        agent.id,
                        STUB_MEMORY_KEY,
                        value,
                        &agent.name,
                        ctx.legacy,
                        ctx.sprint5,
                        &ctx.run_id,
                        Some(step_id),
                    )
                    .map_err(|e| map_memory_error(step_id, e))?;
                prior
            }
            MemoryType::Shared => {
                let owner = state.steps.last().map(|s| s.agent_id).unwrap_or(agent.id);
                let prior = if config.scope == MemoryScope::Workflow {
                    ctx.memory
                        .read_shared(
                            config,
                            owner,
                            STUB_MEMORY_KEY,
                            &agent.name,
                            ctx.legacy,
                            ctx.sprint5,
                            &ctx.run_id,
                            Some(step_id),
                        )
                        .map_err(|e| map_memory_error(step_id, e))?
                } else {
                    None
                };
                ctx.memory
                    .write_shared(
                        agent.id,
                        STUB_MEMORY_KEY,
                        value,
                        config,
                        &agent.name,
                        ctx.legacy,
                        ctx.sprint5,
                        &ctx.run_id,
                        Some(step_id),
                    )
                    .map_err(|e| map_memory_error(step_id, e))?;
                prior
            }
            MemoryType::Persistent => {
                let ns = require_namespace(config, step_id)?;
                let prior = ctx
                    .memory
                    .read_persistent(
                        ns,
                        STUB_MEMORY_KEY,
                        &agent.name,
                        ctx.legacy,
                        ctx.sprint5,
                        &ctx.run_id,
                        Some(step_id),
                    )
                    .map_err(|e| map_memory_error(step_id, e))?;
                ctx.memory
                    .write_persistent(
                        ns,
                        STUB_MEMORY_KEY,
                        value,
                        &agent.name,
                        ctx.legacy,
                        ctx.sprint5,
                        &ctx.run_id,
                        Some(step_id),
                    )
                    .map_err(|e| map_memory_error(step_id, e))?;
                prior
            }
            MemoryType::Vector => {
                let ns = require_namespace(config, step_id)?;
                let prior = ctx
                    .memory
                    .read_vector(
                        ns,
                        STUB_MEMORY_KEY,
                        &agent.name,
                        ctx.legacy,
                        ctx.sprint5,
                        &ctx.run_id,
                        Some(step_id),
                    )
                    .map_err(|e| map_memory_error(step_id, e))?;
                ctx.memory
                    .write_vector(
                        ns,
                        STUB_MEMORY_KEY,
                        value,
                        &agent.name,
                        ctx.legacy,
                        ctx.sprint5,
                        &ctx.run_id,
                        Some(step_id),
                    )
                    .map_err(|e| map_memory_error(step_id, e))?;
                prior
            }
        };
        Ok(bytes_to_note(prior))
    }
}

fn map_tool_error(name: String, step_id: Uuid, err: ToolError) -> RuntimeError {
    RuntimeError::ToolExecutionFailed {
        tool_name: name,
        step_id,
        reason: err.to_string(),
    }
}

const STUB_MEMORY_KEY: &str = "arcflow.stub.context";

fn bytes_to_note(bytes: Option<Vec<u8>>) -> Option<String> {
    bytes.and_then(|b| String::from_utf8(b).ok())
}

fn map_memory_error(step_id: Uuid, err: MemoryError) -> RuntimeError {
    match err {
        MemoryError::InfrastructureUnavailable {
            backend,
            suggestion,
        } => RuntimeError::InfrastructureUnavailable {
            backend,
            suggestion,
            step_id,
        },
        other => RuntimeError::MemoryOperationFailed {
            step_id,
            reason: other.to_string(),
        },
    }
}

fn require_namespace(
    config: &crate::rcs::types::MemoryConfig,
    step_id: Uuid,
) -> Result<&str, RuntimeError> {
    config.namespace.as_deref().filter(|s| !s.is_empty()).ok_or(
        RuntimeError::MemoryOperationFailed {
            step_id,
            reason: "namespace is required for persistent and vector memory".into(),
        },
    )
}

fn map_provider_error(step_id: Uuid, err: ProviderCallError) -> RuntimeError {
    RuntimeError::ProviderCallFailed {
        provider_id: err.provider_id().to_string(),
        step_id,
        reason: err.to_string(),
    }
}

fn emit_provider_error(
    ctx: &mut ExecutionContext<'_, '_>,
    step_id: &str,
    err: &ProviderCallError,
) {
    match err {
        ProviderCallError::RateLimited {
            retry_after_seconds, ..
        } => {
            ctx.sprint5.emit(TraceEventKind::ProviderRateLimited {
                run_id: ctx.run_id.clone(),
                step_id: step_id.to_string(),
                provider_id: err.provider_id().to_string(),
                retry_after_seconds: *retry_after_seconds,
            });
        }
        _ => {
            ctx.sprint5.emit(TraceEventKind::ProviderError {
                run_id: ctx.run_id.clone(),
                step_id: step_id.to_string(),
                provider_id: err.provider_id().to_string(),
                error_code: "provider_call_failed".into(),
                error_message: err.to_string(),
            });
        }
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
