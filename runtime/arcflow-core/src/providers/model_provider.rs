//! ModelProvider trait — exclusive LLM interface (Sprint 6).

use async_trait::async_trait;

use super::error::ProviderCallError;
use super::request::ProviderRequest;
use super::response::{ProviderResponse, ProviderStream};

/// Object-safe provider interface. Implementations: OpenAI, Anthropic, Gemini.
#[async_trait]
pub trait ModelProvider: Send + Sync + 'static {
    fn provider_id(&self) -> &str;
    fn model_id(&self) -> &str;
    async fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderCallError>;
    async fn stream(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderStream, ProviderCallError>;
}
