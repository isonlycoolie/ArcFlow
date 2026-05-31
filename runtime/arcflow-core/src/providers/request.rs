//! Unified provider request types (Sprint 6). Never log or trace prompt content.

use serde_json::Value;

/// JSON-schema tool definition sent to LLM providers (Phase 2-Pro).
#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

/// A single tool invocation requested by the model.
#[derive(Debug, Clone)]
pub struct ToolCallRequest {
    pub id: String,
    pub name: String,
    /// SECURITY: never log or trace.
    pub arguments: String,
}

/// A single message in a provider conversation.
#[derive(Debug, Clone)]
pub struct ProviderMessage {
    pub role: MessageRole,
    /// SECURITY: never log or trace.
    pub content: String,
    /// Tool calls on assistant messages (Phase 2-Pro).
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    /// Tool result correlation id on tool messages (Phase 2-Pro).
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Completion request for any LLM provider.
#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub messages: Vec<ProviderMessage>,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub tools: Vec<ToolSchema>,
}

impl ProviderRequest {
    /// Byte size safe for trace metadata (SEC-2).
    pub fn prompt_size_bytes(&self) -> usize {
        self.messages.iter().map(|m| m.content.len()).sum::<usize>()
            + self
                .system_prompt
                .as_ref()
                .map(|s| s.len())
                .unwrap_or(0)
    }
}
