//! Deterministic mock provider for tool-loop integration tests (Phase 2-Pro).

use async_trait::async_trait;

use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;
use super::model_provider::ModelProvider;
use super::request::{MessageRole, ProviderRequest, ToolCallRequest};
use super::response::{FinishReason, ProviderResponse};

/// Returns a tool call on the first request, then a final answer using tool results.
pub struct MockToolProvider {
    model: String,
}

impl MockToolProvider {
    pub fn new() -> Self {
        Self {
            model: "mock-tool-loop".into(),
        }
    }
}

impl Default for MockToolProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelProvider for MockToolProvider {
    fn provider_id(&self) -> &str {
        "mock"
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderCallError> {
        let tokens = TokenUsage {
            prompt_tokens: request.prompt_size_bytes() as u32,
            completion_tokens: 8,
            total_tokens: request.prompt_size_bytes() as u32 + 8,
        };
        if request.tools.is_empty() {
            let content = request
                .messages
                .iter()
                .rev()
                .find(|m| m.role == MessageRole::User)
                .map(|m| m.content.clone())
                .unwrap_or_default();
            return Ok(ProviderResponse {
                content,
                tokens,
                model_id: self.model.clone(),
                finish_reason: FinishReason::Stop,
                tool_calls: None,
            });
        }
        let has_tool_result = request
            .messages
            .iter()
            .any(|m| m.role == MessageRole::Tool);
        if !has_tool_result {
            let tool = request.tools.first().ok_or_else(|| ProviderCallError::ApiError {
                provider_id: "mock".into(),
                status_code: 400,
                sanitized_message: "no tools".into(),
            })?;
            return Ok(ProviderResponse {
                content: String::new(),
                tokens: tokens.clone(),
                model_id: self.model.clone(),
                finish_reason: FinishReason::Other("tool_calls".into()),
                tool_calls: Some(vec![ToolCallRequest {
                    id: "call_1".into(),
                    name: tool.name.clone(),
                    arguments: r#"{"query":"test"}"#.into(),
                }]),
            });
        }
        let tool_output = request
            .messages
            .iter()
            .filter(|m| m.role == MessageRole::Tool)
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(ProviderResponse {
            content: format!("final answer using {tool_output}"),
            tokens,
            model_id: self.model.clone(),
            finish_reason: FinishReason::Stop,
            tool_calls: None,
        })
    }

    async fn stream(
        &self,
        _request: ProviderRequest,
    ) -> Result<super::response::ProviderStream, ProviderCallError> {
        Err(ProviderCallError::ApiError {
            provider_id: "mock".into(),
            status_code: 501,
            sanitized_message: "streaming not supported on mock provider".into(),
        })
    }
}
