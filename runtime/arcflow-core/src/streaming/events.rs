//! User-facing stream events (Phase 2.1 — SDK-exposed).

use serde::{Deserialize, Serialize};

/// One event emitted on the per-run stream channel when streaming is enabled.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// LLM token chunk (text allowed on stream channel only — not persisted to trace).
    Token { text: String, step_id: String },
    /// Step or graph node execution started.
    StepStart {
        step_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        node_id: Option<String>,
    },
    /// Step finished successfully.
    StepComplete { step_id: String, duration_ms: u64 },
    /// Tool invocation started (argument keys only — not values).
    ToolCall {
        tool_name: String,
        args_keys: Vec<String>,
    },
    /// Step or workflow failure surfaced to the consumer.
    Error {
        code: String,
        message: String,
        step_id: String,
    },
}
