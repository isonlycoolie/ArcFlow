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

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Retry policy applied at workflow or step level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of attempts including the first run.
    pub max_attempts: u32,
    /// Initial backoff delay in milliseconds.
    pub backoff_ms: u64,
    /// Upper bound on backoff delay in milliseconds.
    pub max_backoff_ms: u64,
}

/// Agent memory access configuration (Sprint 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Memory backend kind.
    pub memory_type: MemoryType,
    /// Scope boundary for reads and writes.
    pub scope: MemoryScope,
    /// Optional time-to-live in seconds.
    pub ttl_seconds: Option<u64>,
}

/// External tool specification embedded in agent definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name used for dispatch.
    pub name: String,
    /// JSON Schema describing tool inputs.
    pub input_schema: Value,
    /// Permission strings required to invoke the tool.
    pub permissions: Option<Vec<String>>,
}

/// LLM provider configuration for a run (Sprint 6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider identifier.
    pub provider_id: ProviderId,
    /// Model name passed to the provider API.
    pub model: String,
    /// Environment variable name holding the API key.
    pub api_key_env: String,
    /// Optional provider-specific parameters.
    pub params: Option<Value>,
}

/// Agent role, instructions, and optional tool/memory configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Unique agent identifier.
    pub id: Uuid,
    /// Human-readable agent name.
    pub name: String,
    /// Role label used in prompts and traces.
    pub role: String,
    /// System or task instructions for the agent.
    pub instructions: String,
    /// Tools available to this agent.
    pub tools: Option<Vec<ToolDefinition>>,
    /// Memory configuration for this agent.
    pub memory_config: Option<MemoryConfig>,
}

/// Single step within a workflow definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepDefinition {
    /// Unique step identifier.
    pub id: Uuid,
    /// Agent that executes this step.
    pub agent_id: Uuid,
    /// Execution order relative to other steps.
    pub order: u32,
    /// Optional fallback step when this step fails.
    pub fallback_step_id: Option<Uuid>,
}

/// Complete workflow specification submitted by an SDK.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Unique workflow identifier (UUID v4).
    pub id: Uuid,
    /// Human-readable workflow name (max 256 chars at validation).
    pub name: String,
    /// Steps comprising the workflow.
    pub steps: Vec<StepDefinition>,
    /// Optional default retry policy for all steps.
    pub retry_policy: Option<RetryPolicy>,
}
