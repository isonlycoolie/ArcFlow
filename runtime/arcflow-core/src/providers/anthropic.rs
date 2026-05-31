//! Anthropic messages API provider (Sprint 6 + Phase 2-Pro tools).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::constants::{
    endpoint_from_env, ANTHROPIC_API_ENDPOINT, ANTHROPIC_API_ENDPOINT_ENV, ANTHROPIC_API_KEY_ENV,
    ARCFLOW_USER_AGENT, PROVIDER_REQUEST_TIMEOUT_SECS,
};
use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;
use super::model_provider::ModelProvider;
use super::request::{MessageRole, ProviderRequest, ToolCallRequest};
use super::response::{FinishReason, ProviderResponse, ProviderStream};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

impl AnthropicProvider {
    pub fn new(model: String) -> Result<Self, ProviderCallError> {
        let api_key =
            std::env::var(ANTHROPIC_API_KEY_ENV).map_err(|_| ProviderCallError::NotConfigured {
                provider_id: "anthropic".into(),
                key_env_var: ANTHROPIC_API_KEY_ENV.into(),
            })?;
        Self::with_endpoint(
            model,
            api_key,
            endpoint_from_env(ANTHROPIC_API_ENDPOINT_ENV, ANTHROPIC_API_ENDPOINT),
        )
    }

    pub fn with_endpoint(
        model: String,
        api_key: String,
        endpoint: String,
    ) -> Result<Self, ProviderCallError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(PROVIDER_REQUEST_TIMEOUT_SECS))
            .user_agent(ARCFLOW_USER_AGENT)
            .build()
            .map_err(|e| ProviderCallError::NetworkError {
                provider_id: "anthropic".into(),
                sanitized_message: e.to_string(),
            })?;
        Ok(Self {
            client,
            api_key,
            model,
            endpoint,
        })
    }

    fn build_messages(messages: &[super::request::ProviderMessage]) -> Vec<Value> {
        messages
            .iter()
            .map(|m| match m.role {
                MessageRole::Tool => json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": m.tool_call_id,
                        "content": m.content,
                    }]
                }),
                MessageRole::Assistant if m.tool_calls.is_some() => {
                    let mut blocks: Vec<Value> = Vec::new();
                    if !m.content.is_empty() {
                        blocks.push(json!({"type": "text", "text": m.content}));
                    }
                    for call in m.tool_calls.as_ref().unwrap() {
                        let input: Value =
                            serde_json::from_str(&call.arguments).unwrap_or_else(|_| json!({}));
                        blocks.push(json!({
                            "type": "tool_use",
                            "id": call.id,
                            "name": call.name,
                            "input": input,
                        }));
                    }
                    json!({"role": "assistant", "content": blocks})
                }
                MessageRole::Assistant => json!({"role": "assistant", "content": m.content}),
                _ => json!({"role": "user", "content": m.content}),
            })
            .collect()
    }

    fn build_tools(tools: &[super::request::ToolSchema]) -> Vec<Value> {
        tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                })
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct AnthropicResponse {
    model: String,
    content: Vec<AnthropicBlock>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<Value>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    fn provider_id(&self) -> &str {
        "anthropic"
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderCallError> {
        let mut body = json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
            "messages": Self::build_messages(&request.messages),
        });
        if let Some(system) = &request.system_prompt {
            body["system"] = json!(system);
        }
        if !request.tools.is_empty() {
            body["tools"] = json!(Self::build_tools(&request.tools));
        }
        let response = self
            .client
            .post(&self.endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderCallError::Timeout {
                        provider_id: "anthropic".into(),
                        timeout_secs: PROVIDER_REQUEST_TIMEOUT_SECS,
                    }
                } else {
                    ProviderCallError::NetworkError {
                        provider_id: "anthropic".into(),
                        sanitized_message: "request failed".into(),
                    }
                }
            })?;
        let status = response.status().as_u16();
        if status == 401 {
            return Err(ProviderCallError::AuthenticationFailed {
                provider_id: "anthropic".into(),
                key_env_var: ANTHROPIC_API_KEY_ENV.into(),
            });
        }
        if status == 429 {
            return Err(ProviderCallError::RateLimited {
                provider_id: "anthropic".into(),
                retry_after_seconds: None,
            });
        }
        if !response.status().is_success() {
            return Err(ProviderCallError::ApiError {
                provider_id: "anthropic".into(),
                status_code: status,
                sanitized_message: "api error".into(),
            });
        }
        let parsed: AnthropicResponse = response.json().await.map_err(|e| {
            ProviderCallError::ResponseParseError {
                provider_id: "anthropic".into(),
                reason: e.to_string(),
            }
        })?;
        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();
        for block in parsed.content {
            match block.block_type.as_str() {
                "text" => {
                    if let Some(t) = block.text {
                        text_parts.push(t);
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name)) = (block.id, block.name) {
                        tool_calls.push(ToolCallRequest {
                            id,
                            name,
                            arguments: block
                                .input
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "{}".into()),
                        });
                    }
                }
                _ => {}
            }
        }
        let finish_reason = match parsed.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("tool_use") => FinishReason::Other("tool_use".into()),
            Some("max_tokens") => FinishReason::MaxTokens,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };
        let total = parsed.usage.input_tokens + parsed.usage.output_tokens;
        Ok(ProviderResponse {
            content: text_parts.join(""),
            tokens: TokenUsage {
                prompt_tokens: parsed.usage.input_tokens,
                completion_tokens: parsed.usage.output_tokens,
                total_tokens: total,
            },
            model_id: parsed.model,
            finish_reason,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
        })
    }

    async fn stream(
        &self,
        _request: ProviderRequest,
    ) -> Result<ProviderStream, ProviderCallError> {
        Err(ProviderCallError::ApiError {
            provider_id: "anthropic".into(),
            status_code: 501,
            sanitized_message: "streaming not exposed in Sprint 6 SDK".into(),
        })
    }
}
