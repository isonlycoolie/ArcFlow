//! Anthropic messages API provider (Sprint 6).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::constants::{
    endpoint_from_env, ANTHROPIC_API_ENDPOINT, ANTHROPIC_API_ENDPOINT_ENV, ANTHROPIC_API_KEY_ENV,
    ARCFLOW_USER_AGENT, PROVIDER_REQUEST_TIMEOUT_SECS,
};
use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;
use super::model_provider::ModelProvider;
use super::request::{MessageRole, ProviderRequest};
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
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    temperature: f32,
    system: Option<&'a str>,
    messages: Vec<AnthropicMessage<'a>>,
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
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
    text: String,
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
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .map(|m| AnthropicMessage {
                role: match m.role {
                    MessageRole::Assistant => "assistant",
                    _ => "user",
                },
                content: &m.content,
            })
            .collect();
        let body = AnthropicRequest {
            model: &self.model,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            system: request.system_prompt.as_deref(),
            messages,
        };
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
        let content = parsed
            .content
            .into_iter()
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");
        let finish_reason = match parsed.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::MaxTokens,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };
        let total = parsed.usage.input_tokens + parsed.usage.output_tokens;
        Ok(ProviderResponse {
            content,
            tokens: TokenUsage {
                prompt_tokens: parsed.usage.input_tokens,
                completion_tokens: parsed.usage.output_tokens,
                total_tokens: total,
            },
            model_id: parsed.model,
            finish_reason,
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
