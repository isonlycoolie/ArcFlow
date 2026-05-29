//! Workflow run failures that may include a partial execution record.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::error::RuntimeError;

use super::record::WorkflowExecutionRecord;

/// Outcome when a workflow run does not complete successfully.
#[derive(Debug, Error)]
pub enum WorkflowRunError {
    /// Failed before any step output was committed (validation, missing agent).
    #[error(transparent)]
    Aborted(#[from] RuntimeError),

    /// A step failed after earlier steps completed; `partial` holds committed outputs.
    #[error("workflow run failed: {error}")]
    Failed {
        error: RuntimeError,
        partial: WorkflowExecutionRecord,
    },

    /// Intentional pause awaiting human approval (Phase 1.4 HITL).
    #[error("workflow interrupted for human approval '{approval_key}'")]
    Interrupted {
        approval_key: String,
        expires_at: DateTime<Utc>,
        partial: WorkflowExecutionRecord,
    },
}
