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

/// Machine-readable error codes in `ErrorPayload`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ErrorCode {
    /// Referenced workflow id is not registered.
    WorkflowNotFound,
    /// Workflow definition failed validation.
    InvalidWorkflowDefinition,
    /// A step failed during execution.
    StepExecutionFailed,
    /// LLM provider returned an error.
    ProviderError,
    /// Tool invocation failed.
    ToolExecutionFailed,
    /// Memory subsystem error.
    MemoryError,
    /// Execution exceeded its time budget.
    Timeout,
    /// Provider rate limit reached.
    RateLimited,
    /// Unexpected runtime failure.
    InternalError,
    /// Envelope RCS version is not supported.
    UnsupportedRcsVersion,
}

/// Memory backend kind for agent configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MemoryType {
    /// Ephemeral session-scoped memory.
    Session,
    /// Shared memory across agents in a workflow.
    Shared,
    /// Durable memory with persistence.
    Persistent,
    /// Vector store backed memory.
    Vector,
}

/// Scope boundary for memory access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MemoryScope {
    /// Scoped to a single agent.
    Agent,
    /// Scoped to the current workflow run.
    Workflow,
    /// Global across workflows.
    Global,
}

/// Kind of observability event emitted during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TraceEventKind {
    /// Workflow execution started.
    WorkflowStarted,
    /// A step began running.
    StepStarted,
    /// An agent was invoked for a step.
    AgentInvoked,
    /// Memory read occurred.
    MemoryRead,
    /// Memory write occurred.
    MemoryWrite,
    /// External tool executed.
    ToolExecuted,
    /// Step finished successfully.
    StepCompleted,
    /// Workflow finished successfully.
    WorkflowCompleted,
    /// Step failed.
    StepFailed,
    /// Workflow failed.
    WorkflowFailed,
    /// Step retry attempted.
    RetryAttempted,
}

/// Supported LLM provider identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ProviderId {
    /// OpenAI-compatible provider.
    OpenAI,
    /// Anthropic provider.
    Anthropic,
    /// Google Gemini provider.
    Gemini,
    /// Custom provider implementation.
    Custom,
}
