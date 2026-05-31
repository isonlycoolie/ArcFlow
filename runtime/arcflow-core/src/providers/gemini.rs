//! Google Gemini generateContent provider (Sprint 6 + Phase 2-Pro tools).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::constants::{
    endpoint_from_env, ARCFLOW_USER_AGENT, GEMINI_API_ENDPOINT, GEMINI_API_ENDPOINT_ENV,
    GEMINI_API_KEY_ENV, PROVIDER_REQUEST_TIMEOUT_SECS,
};
use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;
use super::model_provider::ModelProvider;
use super::request::{MessageRole, ProviderRequest, ToolCallRequest};
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

    fn build_contents(messages: &[super::request::ProviderMessage]) -> Vec<GeminiContent> {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::Assistant => "model".into(),
                    _ => "user".into(),
                };
                let parts = match m.role {
                    MessageRole::Tool => vec![GeminiPart {
                        text: None,
                        function_call: None,
                        function_response: Some(GeminiFunctionResponse {
                            name: m.tool_call_id.clone().unwrap_or_default(),
                            response: json!({"result": m.content}),
                        }),
                    }],
                    MessageRole::Assistant if m.tool_calls.is_some() => {
                        let mut parts = Vec::new();
                        if !m.content.is_empty() {
                            parts.push(GeminiPart {
                                text: Some(m.content.clone()),
                                function_call: None,
                                function_response: None,
                            });
                        }
                        for call in m.tool_calls.as_ref().unwrap() {
                            let args: Value =
                                serde_json::from_str(&call.arguments).unwrap_or_else(|_| json!({}));
                            parts.push(GeminiPart {
                                text: None,
                                function_call: Some(GeminiFunctionCall {
                                    name: call.name.clone(),
                                    args,
                                }),
                                function_response: None,
                            });
                        }
                        parts
                    }
                    _ => vec![GeminiPart {
                        text: Some(m.content.clone()),
                        function_call: None,
                        function_response: None,
                    }],
                };
                GeminiContent { role, parts }
            })
            .collect()
    }

    fn build_tools(tools: &[super::request::ToolSchema]) -> Option<Vec<GeminiTool>> {
        if tools.is_empty() {
            return None;
        }
        Some(vec![GeminiTool {
            function_declarations: tools
                .iter()
                .map(|t| GeminiFunctionDeclaration {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.input_schema.clone(),
                })
                .collect(),
        }])
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
}

#[derive(Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiTextPart>,
}

#[derive(Serialize)]
struct GeminiTextPart {
    text: String,
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

#[derive(Serialize, Deserialize, Default)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
}

#[derive(Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: Value,
}

#[derive(Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: Value,
}

#[derive(Serialize)]
struct GeminiTool {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    parameters: Value,
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
        let body = GeminiRequest {
            contents: Self::build_contents(&request.messages),
            system_instruction: request.system_prompt.as_ref().map(|s| GeminiSystemInstruction {
                parts: vec![GeminiTextPart { text: s.clone() }],
            }),
            generation_config: GeminiGenConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
            },
            tools: Self::build_tools(&request.tools),
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
        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();
        for part in candidate.content.parts {
            if let Some(text) = part.text {
                text_parts.push(text);
            }
            if let Some(call) = part.function_call {
                tool_calls.push(ToolCallRequest {
                    id: format!("call_{}", call.name),
                    name: call.name,
                    arguments: call.args.to_string(),
                });
            }
        }
        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::MaxTokens,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };
        let tokens = parsed
            .usage_metadata
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_token_count,
                completion_tokens: u.candidates_token_count,
                total_tokens: u.total_token_count,
            })
            .unwrap_or_default();
        Ok(ProviderResponse {
            content: text_parts.join(""),
            tokens,
            model_id: self.model.clone(),
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
            provider_id: "gemini".into(),
            status_code: 501,
            sanitized_message: "streaming not exposed in Sprint 6 SDK".into(),
        })
    }
}
