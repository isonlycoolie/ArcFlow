//! OpenAI chat completions provider (Sprint 6).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::constants::{
    ARCFLOW_USER_AGENT, OPENAI_API_ENDPOINT, OPENAI_API_KEY_ENV, PROVIDER_REQUEST_TIMEOUT_SECS,
};
use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;
use super::model_provider::ModelProvider;
use super::request::{MessageRole, ProviderRequest};
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
        Self::with_endpoint(model, api_key, OPENAI_API_ENDPOINT.to_string())
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
struct OpenAIChatRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAIMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    content: &'a str,
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
struct OpenAIMessageOwned {
    content: String,
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
        let mut messages = Vec::new();
        if let Some(system) = &request.system_prompt {
            messages.push(OpenAIMessage {
                role: "system",
                content: system,
            });
        }
        for msg in &request.messages {
            let role = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };
            messages.push(OpenAIMessage {
                role,
                content: &msg.content,
            });
        }
        let body = OpenAIChatRequest {
            model: &self.model,
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
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
        Ok(ProviderResponse {
            content: choice.message.content,
            tokens,
            model_id: parsed.model,
            finish_reason,
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
