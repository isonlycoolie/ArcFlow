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
use crate::rcs::types::{AgentDefinition, ExecutionStatus, MemoryScope, MemoryType, ToolExecutionMode};
use crate::state::{ExecutionStepOutput, StateSnapshot};
use crate::streaming::StreamEvent;
use crate::tools::ToolError;
use crate::tracing::events::TraceEventKind;
use crate::tracing::tokens_consumed;
use crate::tracing::TokenUsage;
use crate::workflow::ExecutionContext;

use super::context::{ContextAssembler, ContextExtras};
use super::tool_loop::ToolLoop;
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
        let (memory_note, rag_block) = if let Some(ctx) = ctx.as_mut() {
            self.run_memory_if_configured(agent, step_id, state, run_input, ctx)?
        } else {
            (None, None)
        };

        let policy = ContextAssembler::effective_policy(agent.context.as_ref());
        let graph_state = ctx.as_ref().and_then(|c| c.graph_state.clone());
        let mut extras = ContextExtras {
            memory_note: memory_note.clone(),
            rag_block,
            graph_state,
            ..ContextExtras::default()
        };
        let mut agent_context =
            ContextAssembler::assemble(run_input, state, &policy, &extras);

        if let Some(ctx) = ctx.as_mut() {
            let tool_results = self.run_tools_if_configured(
                agent,
                step_id,
                run_input,
                &agent_context,
                ctx,
            )?;
            if let Some(results) = tool_results {
                extras.tool_results = Some(results);
                agent_context =
                    ContextAssembler::assemble(run_input, state, &policy, &extras);
            }
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
                    let max_attempts = test
                        .stub_responses
                        .get(&stub_key)
                        .and_then(|s| s.fail_times)
                        .map(|fail_times| fail_times.saturating_add(1))
                        .unwrap_or(1);
                    for attempt in 1..=max_attempts {
                        if test.should_fail(&stub_key, attempt) {
                            if attempt >= max_attempts {
                                return Err(RuntimeError::AgentExecutionFailed {
                                    step_id,
                                    reason: format!("test stub failure for {stub_key}"),
                                });
                            }
                            continue;
                        }
                        if let Some(output) = test.stub_output(&stub_key, attempt) {
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
        }

        let step_tokens = if let Some(ctx) = ctx.as_mut() {
            if let Some(provider) = ctx.provider.clone() {
                Some(self.execute_with_provider(
                    agent,
                    step_id,
                    &agent_context,
                    ctx,
                    provider,
                )?)
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
                    "[{role}] processed: {agent_context} (step: {step_id}, prior_steps: {prior})",
                    role = agent.role,
                    agent_context = agent_context,
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
        agent_context: &str,
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
        let use_tool_loop = agent
            .tools
            .as_ref()
            .is_some_and(|t| !t.is_empty())
            && agent
                .tool_execution
                .as_ref()
                .map(|c| c.mode)
                .unwrap_or(ToolExecutionMode::LlmSelect)
                == ToolExecutionMode::LlmSelect;
        if use_tool_loop {
            return ToolLoop::run_sync(
                self,
                agent,
                step_id,
                agent_context,
                ctx,
                provider,
                max_tokens,
                temperature,
            );
        }
        let request =
            build_agent_request(&agent.instructions, agent_context, max_tokens, temperature);
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

        #[cfg(feature = "otel")]
        let _otel_llm = crate::tracing::otel_live::llm_span(
            &ctx.run_id,
            &step_id_str,
            provider.provider_id(),
            provider.model_id(),
        );

        let started = std::time::Instant::now();
        let limit = effective_provider_timeout(ctx);
        let provider_arc = provider.clone();
        let stream_enabled = ctx.stream_tx.is_some();

        if stream_enabled {
            return self.execute_with_provider_stream(
                agent,
                step_id,
                ctx,
                provider,
                &step_id_str,
                started,
                request,
            );
        }

        let provider_call = async {
            if let Some(retry_cfg) = ctx.retry_config.clone() {
                execute_with_retry(
                    || {
                        let req = build_agent_request(
                            &agent.instructions,
                            agent_context,
                            max_tokens,
                            temperature,
                        );
                        let p = provider_arc.clone();
                        async move { p.complete(req).await }
                    },
                    &retry_cfg,
                    &step_id_str,
                    ctx.sprint5,
                )
                .await
                .map_err(|e| match e {
                    RetryError::NonRetryable(err) => {
                        emit_provider_error(ctx, &step_id_str, &err);
                        map_provider_error(step_id, err)
                    }
                    RetryError::Exhausted {
                        attempts_made,
                        last_error_code,
                        error,
                    } => {
                        emit_provider_error(ctx, &step_id_str, &error);
                        RuntimeError::RetryExhausted {
                            step_id: step_id_str.clone(),
                            attempts_made,
                            last_error_code,
                        }
                    }
                })
            } else {
                provider
                    .complete(request)
                    .await
                    .map_err(|err| {
                        emit_provider_error(ctx, &step_id_str, &err);
                        map_provider_error(step_id, err)
                    })
            }
        };

        let result = if let Some(limit) = limit {
            let configured_ms = limit.as_millis() as u64;
            match block_on_provider(async {
                tokio::time::timeout(limit, provider_call).await
            }) {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(err)) if is_deadline_elapsed(&err) => {
                    let elapsed_ms = started.elapsed().as_millis() as u64;
                    let timeout_type = if ctx
                        .workflow_deadline
                        .is_some_and(|d| std::time::Instant::now() >= d)
                    {
                        "workflow"
                    } else {
                        "step"
                    };
                    ctx.sprint5.emit(TraceEventKind::TimeoutEnforced {
                        run_id: ctx.run_id.clone(),
                        step_id: step_id_str.clone(),
                        timeout_type: timeout_type.to_string(),
                        configured_ms,
                        elapsed_ms,
                    });
                    Err(timeout_error_for_context(
                        ctx,
                        &step_id_str,
                        configured_ms,
                        elapsed_ms,
                    ))
                }
                Ok(Err(err)) => Err(err),
                Err(_elapsed) => {
                    let elapsed_ms = started.elapsed().as_millis() as u64;
                    let timeout_type = if ctx
                        .workflow_deadline
                        .is_some_and(|d| std::time::Instant::now() >= d)
                    {
                        "workflow"
                    } else {
                        "step"
                    };
                    ctx.sprint5.emit(TraceEventKind::TimeoutEnforced {
                        run_id: ctx.run_id.clone(),
                        step_id: step_id_str.clone(),
                        timeout_type: timeout_type.to_string(),
                        configured_ms,
                        elapsed_ms,
                    });
                    Err(timeout_error_for_context(
                        ctx,
                        &step_id_str,
                        configured_ms,
                        elapsed_ms,
                    ))
                }
            }
        } else {
            block_on_provider(provider_call)
        };

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
                #[cfg(feature = "otel")]
                crate::tracing::otel_live::record_llm_tokens(
                    provider.provider_id(),
                    provider.model_id(),
                    response.tokens.prompt_tokens,
                    response.tokens.completion_tokens,
                );
                Ok((response.content, response.tokens))
            }
            Err(err) => Err(err),
        }
    }

    fn execute_with_provider_stream(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        ctx: &mut ExecutionContext<'_, '_>,
        provider: std::sync::Arc<dyn crate::providers::ModelProvider>,
        step_id_str: &str,
        started: std::time::Instant,
        request: crate::providers::ProviderRequest,
    ) -> Result<(String, TokenUsage), RuntimeError> {
        use futures_util::StreamExt;

        #[cfg(feature = "otel")]
        let _otel_llm = crate::tracing::otel_live::llm_span(
            &ctx.run_id,
            step_id_str,
            provider.provider_id(),
            provider.model_id(),
        );

        let limit = effective_provider_timeout(ctx);
        let stream = if let Some(limit) = limit {
            let configured_ms = limit.as_millis() as u64;
            match block_on_provider(async {
                tokio::time::timeout(limit, provider.stream(request)).await
            }) {
                Ok(Ok(stream)) => stream,
                Ok(Err(err)) => {
                    emit_provider_error(ctx, step_id_str, &err);
                    return Err(map_provider_error(step_id, err));
                }
                Err(_) => {
                    let elapsed_ms = started.elapsed().as_millis() as u64;
                    ctx.sprint5.emit(TraceEventKind::TimeoutEnforced {
                        run_id: ctx.run_id.clone(),
                        step_id: step_id_str.to_string(),
                        timeout_type: "step".to_string(),
                        configured_ms,
                        elapsed_ms,
                    });
                    return Err(timeout_error_for_context(
                        ctx,
                        step_id_str,
                        configured_ms,
                        elapsed_ms,
                    ));
                }
            }
        } else {
            block_on_provider(provider.stream(request)).map_err(|err| {
                emit_provider_error(ctx, step_id_str, &err);
                map_provider_error(step_id, err)
            })?
        };

        let mut output = String::new();
        let mut tokens = TokenUsage::default();
        let mut prev_tokens = TokenUsage::default();
        let model_id = provider.model_id().to_string();
        let mut stream = stream;

        while let Some(chunk_result) = block_on_provider(stream.next()) {
            match chunk_result {
                Ok(chunk) => {
                    if !chunk.content.is_empty() {
                        ctx.sprint5.emit(TraceEventKind::StreamChunkReceived {
                            run_id: ctx.run_id.clone(),
                            step_id: step_id_str.to_string(),
                            chunk_bytes: chunk.content.len(),
                        });
                        if let Some(tx) = ctx.stream_tx.as_ref() {
                            tx.try_send(StreamEvent::Token {
                                text: chunk.content.clone(),
                                step_id: step_id_str.to_string(),
                            });
                        }
                        output.push_str(&chunk.content);
                    }
                    if let Some(t) = chunk.tokens {
                        let completion_delta = t
                            .completion_tokens
                            .saturating_sub(prev_tokens.completion_tokens);
                        let prompt_delta =
                            t.prompt_tokens.saturating_sub(prev_tokens.prompt_tokens);
                        if completion_delta > 0 || prompt_delta > 0 {
                            ctx.sprint5.emit(TraceEventKind::TokenEmitted {
                                run_id: ctx.run_id.clone(),
                                step_id: step_id_str.to_string(),
                                completion_token_delta: completion_delta,
                                prompt_token_delta: prompt_delta,
                            });
                        }
                        prev_tokens = t.clone();
                        tokens = t;
                    }
                    if chunk.is_final {
                        break;
                    }
                }
                Err(err) => {
                    emit_provider_error(ctx, step_id_str, &err);
                    return Err(map_provider_error(step_id, err));
                }
            }
        }

        let latency_ms = started.elapsed().as_millis() as u64;
        ctx.sprint5.emit(TraceEventKind::ProviderResponseReceived {
            run_id: ctx.run_id.clone(),
            step_id: step_id_str.to_string(),
            provider_id: provider.provider_id().to_string(),
            model_id: model_id.clone(),
            tokens: tokens.clone(),
            latency_ms,
        });
        ctx.sprint5.emit(TraceEventKind::AgentResponseReceived {
            run_id: ctx.run_id.clone(),
            step_id: step_id_str.to_string(),
            agent_name: agent.name.clone(),
            output_size_bytes: output.len(),
        });
        ctx.sprint5.emit(TraceEventKind::TokensConsumed {
            run_id: ctx.run_id.clone(),
            step_id: step_id_str.to_string(),
            agent_name: agent.name.clone(),
            tokens: tokens.clone(),
        });
        #[cfg(feature = "otel")]
        crate::tracing::otel_live::record_llm_tokens(
            provider.provider_id(),
            provider.model_id(),
            tokens.prompt_tokens,
            tokens.completion_tokens,
        );
        Ok((output, tokens))
    }

    fn run_tools_if_configured(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        run_input: &str,
        _agent_context: &str,
        ctx: &mut ExecutionContext<'_, '_>,
    ) -> Result<Option<String>, RuntimeError> {
        let Some(tools) = agent.tools.as_ref() else {
            return Ok(None);
        };
        if tools.is_empty() {
            return Ok(None);
        }
        let mode = agent
            .tool_execution
            .as_ref()
            .map(|c| c.mode)
            .unwrap_or(ToolExecutionMode::LlmSelect);
        if matches!(mode, ToolExecutionMode::LlmSelect) && ctx.provider.is_some() {
            return Ok(None);
        }
        let Some(runtime) = ctx.tool_runtime else {
            return Ok(None);
        };
        let Some(invoker) = ctx.tool_invoker.clone() else {
            return Ok(None);
        };
        let rt = tokio::runtime::Runtime::new().map_err(|e| RuntimeError::ToolExecutionFailed {
            tool_name: "runtime".into(),
            step_id,
            reason: e.to_string(),
        })?;
        let mut lines = Vec::new();
        for def in tools {
            let input = json!({ "message": run_input });
            if let Some(tx) = ctx.stream_tx.as_ref() {
                let args_keys = input
                    .as_object()
                    .map(|obj| obj.keys().cloned().collect())
                    .unwrap_or_default();
                tx.try_send(StreamEvent::ToolCall {
                    tool_name: def.name.clone(),
                    args_keys,
                });
            }
            match rt.block_on(self.execute_tool_call_async(
                crate::providers::ToolCallRequest {
                    id: format!("legacy_{step_id}"),
                    name: def.name.clone(),
                    arguments: serde_json::to_string(&input).unwrap_or_default(),
                },
                tools,
                step_id,
                ctx,
            )) {
                Ok(output) => lines.push(format!("{}: {}", def.name, output)),
                Err(err) => return Err(err),
            }
        }
        if lines.is_empty() {
            Ok(None)
        } else {
            Ok(Some(lines.join("\n")))
        }
    }

    /// Reads prior stub context, writes current input, returns prior value for output annotation.
    fn run_memory_if_configured(
        &self,
        agent: &AgentDefinition,
        step_id: Uuid,
        state: &StateSnapshot,
        run_input: &str,
        ctx: &mut ExecutionContext<'_, '_>,
    ) -> Result<(Option<String>, Option<String>), RuntimeError> {
        let Some(config) = agent.memory_config.as_ref() else {
            return Ok((None, None));
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
                if config.retrieval.is_some() {
                    let ns = require_namespace(config, step_id)?;
                    let top_k = config
                        .retrieval
                        .as_ref()
                        .and_then(|r| r.top_k)
                        .unwrap_or(5) as usize;
                    let query = ContextAssembler::query_for_retrieval(run_input, run_input);
                    let hits = ctx
                        .memory
                        .search_vector(
                            config,
                            &query,
                            top_k,
                            &agent.name,
                            ctx.legacy,
                            ctx.sprint5,
                            &ctx.run_id,
                            Some(step_id),
                        )
                        .map_err(|e| map_memory_error(step_id, e))?;
                    let total_bytes: usize = hits.iter().map(|h| h.len()).sum();
                    ctx.sprint5.emit(TraceEventKind::MemoryRetrieved {
                        run_id: ctx.run_id.clone(),
                        step_id: step_id.to_string(),
                        agent_name: agent.name.clone(),
                        chunk_count: hits.len(),
                        total_bytes,
                    });
                    let rag_block = if hits.is_empty() {
                        None
                    } else {
                        Some(
                            hits.iter()
                                .enumerate()
                                .map(|(i, h)| {
                                    format!(
                                        "[{}] {}",
                                        i + 1,
                                        String::from_utf8_lossy(h)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    };
                    return Ok((None, rag_block));
                }
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
        Ok((bytes_to_note(prior), None))
    }

    pub(crate) async fn execute_tool_call_async(
        &self,
        call: crate::providers::ToolCallRequest,
        tool_defs: &[crate::rcs::types::ToolDefinition],
        step_id: Uuid,
        ctx: &mut ExecutionContext<'_, '_>,
    ) -> Result<String, RuntimeError> {
        let Some(def) = tool_defs.iter().find(|d| d.name == call.name) else {
            return Err(RuntimeError::ToolExecutionFailed {
                tool_name: call.name.clone(),
                step_id,
                reason: "tool not configured on agent".into(),
            });
        };
        let Some(runtime) = ctx.tool_runtime else {
            return Err(RuntimeError::ToolExecutionFailed {
                tool_name: call.name.clone(),
                step_id,
                reason: "tool runtime not configured".into(),
            });
        };
        let Some(invoker) = ctx.tool_invoker.clone() else {
            return Err(RuntimeError::ToolExecutionFailed {
                tool_name: call.name.clone(),
                step_id,
                reason: "tool invoker not configured".into(),
            });
        };
        let input: serde_json::Value =
            serde_json::from_str(&call.arguments).unwrap_or_else(|_| {
                serde_json::json!({ "raw": call.arguments })
            });
        runtime
            .execute_tool(
                &def.name,
                input,
                invoker,
                ctx.legacy,
                ctx.sprint5,
                &ctx.run_id,
                Some(step_id),
            )
            .await
            .map_err(|e| RuntimeError::ToolExecutionFailed {
                tool_name: call.name,
                step_id,
                reason: e.to_string(),
            })
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
            context: None,
            tool_execution: None,
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
