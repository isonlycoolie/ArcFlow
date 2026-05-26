//! Runtime Contract Specification — type definitions.
//!
//! Schema source of truth: `contracts/rcs-v1.schema.json`.

use serde::{Deserialize, Serialize};

/// Dispatch label on every RCS envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MessageType {
    /// Register a workflow definition with the runtime.
    RegisterWorkflow,
    /// Start execution of a registered workflow.
    RunWorkflow,
    /// Final workflow execution outcome.
    WorkflowResult,
    /// Observability trace event emitted during execution.
    TraceEvent,
    /// Protocol or execution error payload.
    Error,
}

/// Lifecycle state for workflows and individual steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExecutionStatus {
    /// Queued but not yet started.
    Pending,
    /// Currently executing.
    Running,
    /// Finished successfully.
    Completed,
    /// Finished with failure.
    Failed,
    /// A retry is in progress after failure.
    Retrying,
    /// Execution was cancelled before completion.
    Cancelled,
}
