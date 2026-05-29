// Runtime Contract Specification — Sprint 1
//
// Types, envelope handling, and protocol errors for SDK ↔ runtime messages.
// Schema source of truth: contracts/normative/rcs/v1.schema.json

pub mod envelope;
pub mod error;
pub mod types;

pub use envelope::MessageEnvelope;
pub use error::RcsError;
pub use types::{
    AgentDefinition, ErrorCode, ErrorPayload, ExecutionStatus, MemoryConfig, MemoryScope,
    MemoryType, MessageType, ProviderConfig, ProviderId, RetryPolicy, RunRequest, RunResult,
    StepDefinition, StepResult, ToolDefinition, TraceEvent, TraceEventKind, WorkflowDefinition,
};
