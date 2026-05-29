//! Runtime Contract Specification — type definitions.
//!
//! Schema source of truth: `contracts/normative/rcs/v1.schema.json`.

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
    /// Paused awaiting human approval (Phase 1.4 HITL).
    Interrupted,
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
    /// Human approval window expired (Phase 1.4 HITL).
    HumanTimeout,
    /// Human rejected the approval request (Phase 1.4 HITL).
    HumanRejected,
    /// Approval key not found for the run (Phase 1.4 HITL).
    ApprovalNotFound,
    /// Approval was already resolved (Phase 1.4 HITL).
    AlreadyApproved,
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
    /// Graph node execution started (Phase 1.1).
    GraphNodeStarted,
    /// Graph node finished (Phase 1.1).
    GraphNodeCompleted,
    /// Graph cycle iteration limit reached (Phase 1.1).
    GraphIterationLimitReached,
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
    /// Namespace for persistent and vector backends.
    pub namespace: Option<String>,
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

/// Human-in-the-loop gate on a step (Phase 1.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitlConfig {
    /// Stable key scoped to the run for approval requests.
    pub approval_key: String,
    /// Wall-clock seconds before the approval expires.
    pub timeout_seconds: u64,
    /// When true, checkpoint and return Interrupted instead of blocking.
    #[serde(default = "default_hitl_interrupt")]
    pub interrupt: bool,
}

fn default_hitl_interrupt() -> bool {
    true
}

/// Human approval payload injected on resume (Phase 1.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalResult {
    pub approved: bool,
    #[serde(default)]
    pub data: Value,
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
    /// Optional human approval gate before agent execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hitl: Option<HitlConfig>,
}

/// Workflow execution strategy (Phase 1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    #[default]
    Linear,
    Graph,
}

/// Node in a graph workflow referencing a step definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub step_ref: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<String>>,
}

/// Directed edge between graph nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Fan-in join for parallel branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinNode {
    pub id: String,
    pub wait_for: Vec<String>,
}

fn default_max_graph_iterations() -> u32 {
    100
}

/// Graph execution topology (Phase 1.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDefinition {
    pub entry_node: String,
    #[serde(default = "default_max_graph_iterations")]
    pub max_iterations: u32,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub join_nodes: Vec<JoinNode>,
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
    /// Linear (default) or graph execution.
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    /// Required when `execution_mode` is graph.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph: Option<GraphDefinition>,
}

/// Request to execute a registered workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRequest {
    /// Registered workflow id to execute.
    pub workflow_id: Uuid,
    /// Caller-supplied input payload as text.
    pub input: String,
    /// Trace id for observability correlation.
    pub trace_id: Uuid,
    /// Optional LLM provider override for this run.
    pub provider_config: Option<ProviderConfig>,
}

/// Outcome of a workflow execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunResult {
    /// Trace id matching the originating run.
    pub trace_id: Uuid,
    /// Overall workflow execution status.
    pub status: ExecutionStatus,
    /// Final workflow output when successful.
    pub output: Option<String>,
    /// Per-step results collected during execution.
    pub steps: Vec<StepResult>,
    /// Error details when status is failed.
    pub error: Option<ErrorPayload>,
}

/// Result of a single step within a workflow run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    /// Step identifier.
    pub step_id: Uuid,
    /// Step execution status.
    pub status: ExecutionStatus,
    /// Step output text when present.
    pub output: Option<String>,
    /// Wall-clock latency in milliseconds.
    pub latency_ms: u64,
    /// Token usage when reported by the provider.
    pub tokens_used: Option<u32>,
}

/// Structured error returned to SDKs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Stable machine-readable error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Step associated with the error when applicable.
    pub step_id: Option<Uuid>,
    /// Whether the caller may retry the operation.
    pub recoverable: bool,
}

/// Single observability event emitted during execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Trace id correlating workflow execution.
    pub trace_id: Uuid,
    /// Event classification.
    pub event_kind: TraceEventKind,
    /// UTC timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Related step id when applicable.
    pub step_id: Option<Uuid>,
    /// Optional structured event payload.
    pub data: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn round_trip<T>(original: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(original).expect("value must serialize to JSON");
        let deserialized: T =
            serde_json::from_str(&json).expect("value must deserialize from JSON");
        assert_eq!(original, &deserialized, "round-trip must preserve value");
        deserialized
    }

    #[test]
    fn message_type_round_trip() {
        round_trip(&MessageType::RunWorkflow);
    }

    #[test]
    fn execution_status_round_trip() {
        round_trip(&ExecutionStatus::Running);
    }

    #[test]
    fn error_code_round_trip() {
        round_trip(&ErrorCode::WorkflowNotFound);
    }

    #[test]
    fn memory_type_round_trip() {
        round_trip(&MemoryType::Session);
    }

    #[test]
    fn memory_scope_round_trip() {
        round_trip(&MemoryScope::Workflow);
    }

    #[test]
    fn trace_event_kind_round_trip() {
        round_trip(&TraceEventKind::StepStarted);
    }

    #[test]
    fn provider_id_round_trip() {
        round_trip(&ProviderId::Anthropic);
    }

    #[test]
    fn retry_policy_round_trip() {
        round_trip(&RetryPolicy {
            max_attempts: 3,
            backoff_ms: 100,
            max_backoff_ms: 5_000,
        });
    }

    #[test]
    fn memory_config_round_trip() {
        round_trip(&MemoryConfig {
            memory_type: MemoryType::Vector,
            scope: MemoryScope::Agent,
            namespace: Some("ns".into()),
            ttl_seconds: Some(3600),
        });
    }

    #[test]
    fn tool_definition_round_trip() {
        round_trip(&ToolDefinition {
            name: "search".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            permissions: Some(vec!["read".to_string()]),
        });
    }

    #[test]
    fn provider_config_round_trip() {
        round_trip(&ProviderConfig {
            provider_id: ProviderId::OpenAI,
            model: "gpt-4".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            params: Some(serde_json::json!({"temperature": 0.2})),
        });
    }

    #[test]
    fn workflow_definition_round_trip() {
        round_trip(&WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "test-workflow".to_string(),
            steps: vec![StepDefinition {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                order: 1,
                fallback_step_id: None,
                hitl: None,
            }],
            retry_policy: None,
            execution_mode: ExecutionMode::Linear,
            graph: None,
        });
    }

    #[test]
    fn agent_definition_round_trip() {
        round_trip(&AgentDefinition {
            id: Uuid::new_v4(),
            name: "researcher".to_string(),
            role: "research".to_string(),
            instructions: "Find sources".to_string(),
            tools: None,
            memory_config: None,
        });
    }

    #[test]
    fn step_definition_round_trip() {
        round_trip(&StepDefinition {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            order: 2,
            fallback_step_id: Some(Uuid::new_v4()),
            hitl: None,
        });
    }
