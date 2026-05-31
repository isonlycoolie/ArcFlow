//! Unified provider response types (Sprint 6).

use std::pin::Pin;

use futures_util::Stream;
use serde::{Deserialize, Serialize};

use crate::tracing::types::TokenUsage;

use super::error::ProviderCallError;

use super::request::ToolCallRequest;

/// Why the model stopped generating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    MaxTokens,
    ContentFilter,
    Other(String),
}

/// Completed provider response.
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    /// SECURITY: never log or trace.
    pub content: String,
    pub tokens: TokenUsage,
    pub model_id: String,
    pub finish_reason: FinishReason,
    /// Tool calls when the model requests tool execution (Phase 2-Pro).
    pub tool_calls: Option<Vec<ToolCallRequest>>,
}

impl ProviderResponse {
    pub fn content_size_bytes(&self) -> usize {
        self.content.len()
    }
}

/// One chunk in a streaming response.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
    pub tokens: Option<TokenUsage>,
}

pub type ProviderStream =
    Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderCallError>> + Send>>;
