//! Builds `ModelProvider` instances from RCS config.

use std::sync::Arc;

use crate::constants::{PROVIDER_DEFAULT_MAX_TOKENS, PROVIDER_DEFAULT_TEMPERATURE};
use crate::rcs::types::{ProviderConfig, ProviderId};

use super::anthropic::AnthropicProvider;
use super::error::ProviderCallError;
use super::gemini::GeminiProvider;
use super::model_provider::ModelProvider;
use super::openai::OpenAIProvider;
use super::request::ProviderRequest;

pub struct ProviderRuntime;

impl ProviderRuntime {
    pub fn from_config(
        config: &ProviderConfig,
    ) -> Result<Arc<dyn ModelProvider>, ProviderCallError> {
        match config.provider_id {
            ProviderId::OpenAI => Ok(Arc::new(OpenAIProvider::new(config.model.clone())?)),
            ProviderId::Anthropic => Ok(Arc::new(AnthropicProvider::new(config.model.clone())?)),
            ProviderId::Gemini => Ok(Arc::new(GeminiProvider::new(config.model.clone())?)),
            ProviderId::Custom => Err(ProviderCallError::ApiError {
                provider_id: "custom".into(),
                status_code: 400,
                sanitized_message: "custom providers not supported in Sprint 6".into(),
            }),
        }
    }
}

/// Builds a provider request from agent instructions and run input.
pub fn build_agent_request(
    instructions: &str,
    run_input: &str,
    max_tokens: u32,
    temperature: f32,
) -> ProviderRequest {
    ProviderRequest {
        messages: vec![super::request::ProviderMessage {
            role: super::request::MessageRole::User,
            content: run_input.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }],
        system_prompt: Some(instructions.to_string()),
        max_tokens,
        temperature,
        tools: vec![],
    }
}

pub fn default_max_tokens() -> u32 {
    PROVIDER_DEFAULT_MAX_TOKENS
}

pub fn default_temperature() -> f32 {
    PROVIDER_DEFAULT_TEMPERATURE
}
