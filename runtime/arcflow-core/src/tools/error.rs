//! Tool runtime errors.

use thiserror::Error;
use uuid::Uuid;

/// Failures from tool registration, validation, or execution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ToolError {
    /// Tool name is not registered for this run.
    #[error("tool '{name}' is not registered")]
    NotRegistered { name: String },

    /// Duplicate tool name at registration.
    #[error("tool '{name}' is already registered")]
    DuplicateName { name: String },

    /// JSON Schema validation failed.
    #[error("tool input validation failed for '{name}': {reason}")]
    ValidationFailed { name: String, reason: String },

    /// User executor returned an error.
    #[error("tool '{name}' execution failed: {reason}")]
    ExecutionFailed {
        name: String,
        step_id: Option<Uuid>,
        reason: String,
    },

    /// Wall-clock timeout exceeded.
    #[error("tool '{name}' timed out after {timeout_secs} seconds")]
    Timeout { name: String, timeout_secs: u64 },
}
