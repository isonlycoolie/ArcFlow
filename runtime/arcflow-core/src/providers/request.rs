//! Unified provider request types (Sprint 6). Never log or trace prompt content.

/// A single message in a provider conversation.
#[derive(Debug, Clone)]
pub struct ProviderMessage {
    pub role: MessageRole,
    /// SECURITY: never log or trace.
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Completion request for any LLM provider.
#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub messages: Vec<ProviderMessage>,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
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
