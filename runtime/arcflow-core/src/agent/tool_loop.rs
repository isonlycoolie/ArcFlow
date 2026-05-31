//! LLM tool loop — function calling with iteration cap (Phase 2-Pro).

use std::sync::Arc;

use serde_json::Value;
use uuid::Uuid;

use crate::error::RuntimeError;
use crate::providers::{
    MessageRole, ModelProvider, ProviderMessage, ProviderRequest, ToolCallRequest, ToolSchema,
};
use crate::rcs::types::{AgentDefinition, ToolDefinition, ToolExecutionConfig};
use crate::tracing::TokenUsage;
use crate::workflow::ExecutionContext;

use super::runtime::AgentRuntime;

pub struct ToolLoop;

impl ToolLoop {
    pub fn max_iterations(agent: &AgentDefinition) -> u32 {
        agent
            .tool_execution
            .as_ref()
            .map(|c| c.max_iterations)
            .unwrap_or_else(|| ToolExecutionConfig::default().max_iterations)
            .clamp(1, 20)
    }

    pub fn tool_schemas(tools: &[ToolDefinition]) -> Vec<ToolSchema> {
        tools
            .iter()
            .map(|t| ToolSchema {
                name: t.name.clone(),
                description: None,
                input_schema: t.input_schema.clone(),
            })
            .collect()
    }

    pub fn run_sync(
        agent_runtime: &AgentRuntime,
        agent: &AgentDefinition,
        step_id: Uuid,
        agent_context: &str,
        ctx: &mut ExecutionContext<'_, '_>,
        provider: Arc<dyn ModelProvider>,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<(String, TokenUsage), RuntimeError> {
        let tool_defs = agent
            .tools
            .as_ref()
            .filter(|t| !t.is_empty())
            .ok_or_else(|| RuntimeError::AgentExecutionFailed {
                step_id,
                reason: "tool loop invoked without tools".into(),
            })?;
        let schemas = Self::tool_schemas(tool_defs);
        let max_iter = Self::max_iterations(agent);
        let rt = tokio::runtime::Runtime::new().map_err(|e| RuntimeError::ToolExecutionFailed {
            tool_name: "runtime".into(),
            step_id,
            reason: e.to_string(),
        })?;
        rt.block_on(Self::run_async(
            agent_runtime,
            agent,
            step_id,
            agent_context,
            ctx,
            provider,
            max_tokens,
            temperature,
            schemas,
            tool_defs,
            max_iter,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_async(
        agent_runtime: &AgentRuntime,
        agent: &AgentDefinition,
        step_id: Uuid,
        agent_context: &str,
        ctx: &mut ExecutionContext<'_, '_>,
        provider: Arc<dyn ModelProvider>,
        max_tokens: u32,
        temperature: f32,
        schemas: Vec<ToolSchema>,
        tool_defs: &[ToolDefinition],
        max_iter: u32,
    ) -> Result<(String, TokenUsage), RuntimeError> {
        let mut messages = vec![ProviderMessage {
            role: MessageRole::User,
            content: agent_context.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }];
        let mut total_tokens = TokenUsage::default();
        let step_id_str = step_id.to_string();

        for _ in 0..max_iter {
            let request = ProviderRequest {
                messages: messages.clone(),
                system_prompt: Some(agent.instructions.clone()),
                max_tokens,
                temperature,
                tools: schemas.clone(),
            };
            ctx.sprint5
                .emit(crate::tracing::events::TraceEventKind::ProviderRequestSent {
                    run_id: ctx.run_id.clone(),
                    step_id: step_id_str.clone(),
                    provider_id: provider.provider_id().to_string(),
                    model_id: provider.model_id().to_string(),
                    max_tokens: request.max_tokens,
                    prompt_size_bytes: request.prompt_size_bytes(),
                });
            let started = std::time::Instant::now();
            let response = provider.complete(request).await.map_err(|e| {
                RuntimeError::ProviderCallFailed {
                    provider_id: provider.provider_id().to_string(),
                    step_id,
                    reason: e.to_string(),
                }
            })?;
            let latency_ms = started.elapsed().as_millis() as u64;
            ctx.sprint5
                .emit(crate::tracing::events::TraceEventKind::ProviderResponseReceived {
                    run_id: ctx.run_id.clone(),
                    step_id: step_id_str.clone(),
                    provider_id: provider.provider_id().to_string(),
                    model_id: response.model_id.clone(),
                    tokens: response.tokens.clone(),
                    latency_ms,
                });
            total_tokens.prompt_tokens += response.tokens.prompt_tokens;
            total_tokens.completion_tokens += response.tokens.completion_tokens;
            total_tokens.total_tokens += response.tokens.total_tokens;

            if let Some(calls) = response.tool_calls.filter(|c| !c.is_empty()) {
                messages.push(ProviderMessage {
                    role: MessageRole::Assistant,
                    content: response.content,
                    tool_calls: Some(calls.clone()),
                    tool_call_id: None,
                });
                for call in calls {
                    let call_id = call.id.clone();
                    let output = agent_runtime
                        .execute_tool_call_async(call, tool_defs, step_id, ctx)
                        .await?;
                    messages.push(ProviderMessage {
                        role: MessageRole::Tool,
                        content: output,
                        tool_calls: None,
                        tool_call_id: Some(call_id),
                    });
                }
                continue;
            }
            return Ok((response.content, total_tokens));
        }
        Err(RuntimeError::ToolExecutionFailed {
            tool_name: "tool_loop".into(),
            step_id,
            reason: format!("exceeded max tool iterations ({max_iter})"),
        })
    }
}
