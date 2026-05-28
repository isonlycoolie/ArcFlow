//! Named runtime limits (Sprint 5 observability and shared caps).

/// Maximum trace events retained per workflow run before dropping oldest events.
pub const MAX_TRACE_EVENTS_PER_RUN: u32 = 10_000;

/// Maximum completed run traces held in the in-process store at once.
pub const MAX_CONCURRENT_TRACES: usize = 100;

/// OTLP export timeout in milliseconds (best-effort after run completes).
pub const OTLP_EXPORT_TIMEOUT_MS: u64 = 5_000;

/// Environment variable names for provider API keys (SEC-1).
pub const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
pub const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";
pub const GEMINI_API_KEY_ENV: &str = "GEMINI_API_KEY";

/// Default provider HTTP endpoints.
pub const OPENAI_API_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
pub const ANTHROPIC_API_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
pub const GEMINI_API_ENDPOINT: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";

pub const PROVIDER_REQUEST_TIMEOUT_SECS: u64 = 120;
pub const PROVIDER_DEFAULT_MAX_TOKENS: u32 = 4096;
pub const PROVIDER_DEFAULT_TEMPERATURE: f32 = 0.7;
pub const PROVIDER_MAX_STREAM_TOKENS: u32 = 8192;
pub const PROVIDER_STREAM_TIMEOUT_SECS: u64 = 120;
pub const ARCFLOW_USER_AGENT: &str = concat!("arcflow/", env!("CARGO_PKG_VERSION"));
