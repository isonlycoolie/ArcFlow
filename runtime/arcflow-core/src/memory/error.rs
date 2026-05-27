//! Memory subsystem errors.

use thiserror::Error;

/// Failures from memory configuration or operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemoryError {
    /// Key not found for read.
    #[error("memory key not found")]
    NotFound,

    /// Session key belongs to another agent.
    #[error("session memory is isolated per agent")]
    SessionIsolationViolation,

    /// Shared read without workflow scope.
    #[error("shared memory requires workflow scope on agent config")]
    ScopeDenied,

    /// Persistent or vector backend unavailable.
    #[error("infrastructure unavailable: {backend} — {suggestion}")]
    InfrastructureUnavailable {
        backend: String,
        suggestion: String,
    },

    /// Namespace required for durable backends.
    #[error("namespace is required for persistent and vector memory")]
    NamespaceRequired,

    /// Backend returned an error.
    #[error("memory operation failed: {reason}")]
    OperationFailed { reason: String },
}
