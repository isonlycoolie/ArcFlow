//! Google Gemini generateContent provider (Sprint 6).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::constants::{
    endpoint_from_env, ARCFLOW_USER_AGENT, GEMINI_API_ENDPOINT, GEMINI_API_ENDPOINT_ENV,
    GEMINI_API_KEY_ENV, PROVIDER_REQUEST_TIMEOUT_SECS,
};
use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;
use super::model_provider::ModelProvider;
use super::request::{MessageRole, ProviderRequest};
use super::response::{FinishReason, ProviderResponse, ProviderStream};

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiProvider {
    pub fn new(model: String) -> Result<Self, ProviderCallError> {
        let api_key = std::env::var(GEMINI_API_KEY_ENV).map_err(|_| ProviderCallError::NotConfigured {
            provider_id: "gemini".into(),
            key_env_var: GEMINI_API_KEY_ENV.into(),
        })?;
        Self::with_base_url(
            model,
            api_key,
            endpoint_from_env(GEMINI_API_ENDPOINT_ENV, GEMINI_API_ENDPOINT),
        )
    }

    pub fn with_base_url(
        model: String,
        api_key: String,
        base_url: String,
    ) -> Result<Self, ProviderCallError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(PROVIDER_REQUEST_TIMEOUT_SECS))
            .user_agent(ARCFLOW_USER_AGENT)
            .build()
            .map_err(|e| ProviderCallError::NetworkError {
                provider_id: "gemini".into(),
                sanitized_message: e.to_string(),
            })?;
        Ok(Self {
            client,
            api_key,
            model,
            base_url,
        })
    }

    fn url(&self) -> String {
        format!("{}/{}:generateContent", self.base_url, self.model)
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction")]
    system_instruction: Option<GeminiPart>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenConfig,
}

#[derive(Serialize)]
struct GeminiGenConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiUsage {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    fn provider_id(&self) -> &str {
        "gemini"
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderCallError> {
        let contents: Vec<GeminiContent> = request
            .messages
            .iter()
            .map(|m| GeminiContent {
                role: match m.role {
                    MessageRole::Assistant => "model".into(),
                    _ => "user".into(),
                },
                parts: vec![GeminiPart {
                    text: m.content.clone(),
                }],
            })
            .collect();
        let body = GeminiRequest {
            contents,
            system_instruction: request
                .system_prompt
                .as_ref()
                .map(|s| GeminiPart { text: s.clone() }),
            generation_config: GeminiGenConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
            },
        };
        let response = self
            .client
            .post(self.url())
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ProviderCallError::Timeout {
                        provider_id: "gemini".into(),
                        timeout_secs: PROVIDER_REQUEST_TIMEOUT_SECS,
                    }
                } else {
                    ProviderCallError::NetworkError {
                        provider_id: "gemini".into(),
                        sanitized_message: "request failed".into(),
                    }
                }
            })?;
        let status = response.status().as_u16();
        if status == 401 || status == 403 {
            return Err(ProviderCallError::AuthenticationFailed {
                provider_id: "gemini".into(),
                key_env_var: GEMINI_API_KEY_ENV.into(),
            });
        }
        if status == 429 {
            return Err(ProviderCallError::RateLimited {
                provider_id: "gemini".into(),
                retry_after_seconds: None,
            });
        }
        if !response.status().is_success() {
            return Err(ProviderCallError::ApiError {
                provider_id: "gemini".into(),
                status_code: status,
                sanitized_message: "api error".into(),
            });
        }
        let parsed: GeminiResponse = response.json().await.map_err(|e| {
            ProviderCallError::ResponseParseError {
                provider_id: "gemini".into(),
                reason: e.to_string(),
            }
        })?;
        let candidate = parsed.candidates.into_iter().next().ok_or_else(|| {
            ProviderCallError::ResponseParseError {
                provider_id: "gemini".into(),
                reason: "no candidates".into(),
            }
        })?;
        let content = candidate
            .content
            .parts
            .into_iter()
            .map(|p| p.text)
            .collect::<Vec<_>>()
            .join("");
        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::MaxTokens,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };
        let tokens = parsed.usage_metadata.map(|u| TokenUsage {
            prompt_tokens: u.prompt_token_count,
            completion_tokens: u.candidates_token_count,
            total_tokens: u.total_token_count,
        }).unwrap_or_default();
        Ok(ProviderResponse {
            content,
            tokens,
            model_id: self.model.clone(),
            finish_reason,
        })
    }

    async fn stream(
        &self,
        _request: ProviderRequest,
    ) -> Result<ProviderStream, ProviderCallError> {
        Err(ProviderCallError::ApiError {
            provider_id: "gemini".into(),
            status_code: 501,
            sanitized_message: "streaming not exposed in Sprint 6 SDK".into(),
        })
    }
}
