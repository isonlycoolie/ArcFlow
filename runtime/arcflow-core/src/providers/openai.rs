//! OpenAI chat completions provider (Sprint 6).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::constants::{
    endpoint_from_env, ARCFLOW_USER_AGENT, OPENAI_API_ENDPOINT, OPENAI_API_ENDPOINT_ENV,
    OPENAI_API_KEY_ENV, PROVIDER_REQUEST_TIMEOUT_SECS,
};
use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;
use super::model_provider::ModelProvider;
use super::request::{MessageRole, ProviderRequest, ToolCallRequest, ToolSchema};
use super::response::{FinishReason, ProviderResponse, ProviderStream};

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

impl OpenAIProvider {
    pub fn new(model: String) -> Result<Self, ProviderCallError> {
        let api_key = std::env::var(OPENAI_API_KEY_ENV).map_err(|_| ProviderCallError::NotConfigured {
            provider_id: "openai".into(),
            key_env_var: OPENAI_API_KEY_ENV.into(),
        })?;
        Self::with_endpoint(
            model,
            api_key,
            endpoint_from_env(OPENAI_API_ENDPOINT_ENV, OPENAI_API_ENDPOINT),
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
                provider_id: "openai".into(),
                sanitized_message: e.to_string(),
            })?;
        Ok(Self {
            client,
            api_key,
            model,
            endpoint,
        })
    }

    fn map_http_error(&self, status: u16, code: Option<&str>) -> ProviderCallError {
        if status == 401 {
            return ProviderCallError::AuthenticationFailed {
                provider_id: "openai".into(),
                key_env_var: OPENAI_API_KEY_ENV.into(),
            };
        }
        if status == 429 {
            return ProviderCallError::RateLimited {
                provider_id: "openai".into(),
                retry_after_seconds: None,
            };
        }
        ProviderCallError::ApiError {
            provider_id: "openai".into(),
            status_code: status,
            sanitized_message: code.unwrap_or("api error").to_string(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<serde_json::Value>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAIToolDef>,
}

#[derive(Serialize)]
struct OpenAIToolDef {
    r#type: &'static str,
    function: OpenAIFunctionDef,
}

#[derive(Serialize)]
struct OpenAIFunctionDef {
    name: String,
    parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIMessageOwned {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Deserialize)]
struct OpenAIToolCall {
    id: String,
    function: OpenAIToolFunction,
}

#[derive(Deserialize)]
struct OpenAIToolFunction {
    name: String,
    arguments: String,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageOwned,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    fn provider_id(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderCallError> {
        let mut messages: Vec<serde_json::Value> = Vec::new();
        if let Some(system) = &request.system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system,
            }));
        }
        for msg in &request.messages {
            match msg.role {
                MessageRole::System => messages.push(serde_json::json!({
                    "role": "system",
                    "content": msg.content,
                })),
                MessageRole::User => messages.push(serde_json::json!({
                    "role": "user",
                    "content": msg.content,
                })),
                MessageRole::Assistant => {
                    if let Some(calls) = &msg.tool_calls {
                        let tool_calls: Vec<serde_json::Value> = calls
                            .iter()
                            .map(|c| {
                                serde_json::json!({
                                    "id": c.id,
                                    "type": "function",
                                    "function": {
                                        "name": c.name,
                                        "arguments": c.arguments,
                                    }
                                })
                            })
                            .collect();
                        messages.push(serde_json::json!({
                            "role": "assistant",
                            "content": msg.content,
                            "tool_calls": tool_calls,
                        }));
                    } else {
                        messages.push(serde_json::json!({
                            "role": "assistant",
                            "content": msg.content,
                        }));
                    }
                }
                MessageRole::Tool => messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": msg.tool_call_id,
                    "content": msg.content,
                })),
            }
        }
        let tools: Vec<OpenAIToolDef> = request
            .tools
            .iter()
            .map(|t| OpenAIToolDef {
                r#type: "function",
                function: OpenAIFunctionDef {
                    name: t.name.clone(),
                    parameters: t.input_schema.clone(),
                    description: t.description.clone(),
                },
            })
            .collect();
        let body = OpenAIChatRequest {
            model: self.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            tools,
        };
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderCallError::Timeout {
                        provider_id: "openai".into(),
                        timeout_secs: PROVIDER_REQUEST_TIMEOUT_SECS,
                    }
                } else {
                    ProviderCallError::NetworkError {
                        provider_id: "openai".into(),
                        sanitized_message: "request failed".into(),
                    }
                }
            })?;
        let status = response.status().as_u16();
        if !response.status().is_success() {
            return Err(self.map_http_error(status, None));
        }
        let parsed: OpenAIChatResponse = response.json().await.map_err(|e| {
            ProviderCallError::ResponseParseError {
                provider_id: "openai".into(),
                reason: e.to_string(),
            }
        })?;
        let choice = parsed.choices.into_iter().next().ok_or_else(|| {
            ProviderCallError::ResponseParseError {
                provider_id: "openai".into(),
                reason: "no choices".into(),
            }
        })?;
        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::MaxTokens,
            Some("content_filter") => FinishReason::ContentFilter,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };
        let tokens = parsed
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_default();
        let tool_calls = choice.message.tool_calls.map(|calls| {
            calls
                .into_iter()
                .map(|c| ToolCallRequest {
                    id: c.id,
                    name: c.function.name,
                    arguments: c.function.arguments,
                })
                .collect()
        });
        Ok(ProviderResponse {
            content: choice.message.content.unwrap_or_default(),
            tokens,
            model_id: parsed.model,
            finish_reason,
            tool_calls,
        })
    }

    async fn stream(
        &self,
        _request: ProviderRequest,
    ) -> Result<ProviderStream, ProviderCallError> {
        Err(ProviderCallError::ApiError {
            provider_id: "openai".into(),
            status_code: 501,
            sanitized_message: "streaming not exposed in Sprint 6 SDK".into(),
        })
    }
}
