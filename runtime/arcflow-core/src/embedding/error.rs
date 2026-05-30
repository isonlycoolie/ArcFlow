//! Embedding provider errors (Phase 1.5).

use thiserror::Error;

/// Failures from embedding providers and registry resolution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EmbeddingError {
    /// Provider spec string is invalid or unsupported.
    #[error("invalid embedding provider spec: {reason}")]
    InvalidSpec { reason: String },

    /// Remote provider blocked by privacy mode.
    #[error("remote embedding provider '{provider}' blocked by ARCFLOW_EMBEDDING_LOCAL_ONLY")]
    LocalOnlyViolation { provider: String },

    /// Required API key or configuration is missing.
    #[error("embedding provider not configured: {reason}")]
    NotConfigured { reason: String },

    /// HTTP or network failure talking to a remote provider.
    #[error("embedding request failed: {reason}")]
    RequestFailed { reason: String },

    /// Provider returned an empty embedding batch.
    #[error("embedding provider returned no vectors")]
    EmptyBatch,

    /// Provider response could not be parsed.
    #[error("embedding response parse error: {reason}")]
    ParseError { reason: String },
}
