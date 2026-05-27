//! Typed errors for synchronous runtime execution (Sprint 2).

use thiserror::Error;
use uuid::Uuid;

/// Failure when committing a step result into [`crate::state::StateEngine`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
    /// A step id was committed more than once.
    #[error("step '{step_id}' has already been committed")]
    DuplicateCommit { step_id: Uuid },
}

/// Top-level failures from [`crate::workflow::WorkflowEngine`].
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Step references an agent id missing from the registration map.
    #[error("agent '{agent_id}' referenced in step '{step_id}' was not registered")]
    AgentNotFound { agent_id: Uuid, step_id: Uuid },

    /// [`crate::state::StateEngine::commit`] rejected the output.
    #[error("state commit failed for step '{step_id}': {reason}")]
    StateCommitFailed { step_id: Uuid, reason: String },

    /// Definition failed static validation before execution.
    #[error("workflow definition is invalid: {reason}")]
    InvalidWorkflowDefinition { reason: String },

    /// Stub agent execution failed (reserved for future non-stub paths).
    #[error("agent execution failed for step '{step_id}': {reason}")]
    AgentExecutionFailed { step_id: Uuid, reason: String },

    /// Tool invocation failed during a step.
    #[error("tool '{tool_name}' failed for step '{step_id}': {reason}")]
    ToolExecutionFailed {
        tool_name: String,
        step_id: Uuid,
        reason: String,
    },

    /// Memory operation failed.
    #[error("memory operation failed: {reason}")]
    MemoryOperationFailed { reason: String },
}
