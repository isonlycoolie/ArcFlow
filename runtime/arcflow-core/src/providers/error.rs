//! Provider API failure types (Sprint 6).

use thiserror::Error;

/// Errors from LLM provider HTTP calls. Messages must never contain credentials or prompts.
#[derive(Debug, Error)]
pub enum ProviderCallError {
    #[error(
        "provider '{provider_id}' authentication failed — check that {key_env_var} is set correctly"
    )]
    AuthenticationFailed {
        provider_id: String,
        key_env_var: String,
    },

    #[error("provider '{provider_id}' rate limit reached — retry after {retry_after_seconds:?}s")]
    RateLimited {
        provider_id: String,
        retry_after_seconds: Option<u64>,
    },

    #[error("provider '{provider_id}' request timed out after {timeout_secs}s")]
    Timeout {
        provider_id: String,
        timeout_secs: u64,
    },

    #[error("provider '{provider_id}' returned error {status_code}: {sanitized_message}")]
    ApiError {
        provider_id: String,
        status_code: u16,
        sanitized_message: String,
    },

    #[error("provider '{provider_id}' response could not be parsed: {reason}")]
    ResponseParseError { provider_id: String, reason: String },

    #[error("provider '{provider_id}' is not configured — set {key_env_var} environment variable")]
    NotConfigured {
        provider_id: String,
        key_env_var: String,
    },

    #[error("provider '{provider_id}' content was filtered")]
    ContentFiltered { provider_id: String },

    #[error("provider '{provider_id}' network error: {sanitized_message}")]
    NetworkError {
        provider_id: String,
        sanitized_message: String,
    },
}

impl ProviderCallError {
    /// Stable provider id for trace events.
    pub fn provider_id(&self) -> &str {
        match self {
            Self::AuthenticationFailed { provider_id, .. }
            | Self::RateLimited { provider_id, .. }
            | Self::Timeout { provider_id, .. }
            | Self::ApiError { provider_id, .. }
            | Self::ResponseParseError { provider_id, .. }
            | Self::NotConfigured { provider_id, .. }
            | Self::ContentFiltered { provider_id }
            | Self::NetworkError { provider_id, .. } => provider_id,
        }
    }

    /// Whether Sprint 7 retry engine may retry this error.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::Timeout { .. } | Self::NetworkError { .. }
        )
    }
}
