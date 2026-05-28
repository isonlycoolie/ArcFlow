//! Core trace aggregate types (Sprint 5). See TRACE-EVENT-SCHEMA-v1.md.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::events::TraceEventKind;

/// Token counts for a step or run (never token strings).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    /// Sums two usage records.
    pub fn add(&self, other: &TokenUsage) -> TokenUsage {
        TokenUsage {
            prompt_tokens: self.prompt_tokens + other.prompt_tokens,
            completion_tokens: self.completion_tokens + other.completion_tokens,
            total_tokens: self.total_tokens + other.total_tokens,
        }
    }
}

/// Single structured trace event (Sprint 5 envelope).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub trace_id: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub kind: TraceEventKind,
}

impl TraceEvent {
    #[allow(dead_code)] // Sprint 5 Phase 3: TraceEventEmitter::emit
    pub(crate) fn new(trace_id: String, sequence: u64, kind: TraceEventKind) -> Self {
        Self {
            trace_id,
            timestamp: Utc::now(),
            sequence,
            kind,
        }
    }
}

/// Workflow-level execution status in a completed trace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Completed,
    Failed,
    Partial,
}

/// Complete trace for one workflow run (assembled by ExecutionTraceBuilder).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub run_id: String,
    pub workflow_name: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub total_tokens: TokenUsage,
    pub steps: Vec<StepTrace>,
    pub events: Vec<TraceEvent>,
    pub is_complete: bool,
    pub events_dropped: u32,
}

/// Per-step summary (populated by builder in Phase 4).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepTrace {
    pub step_index: usize,
    pub step_id: String,
    pub agent_name: String,
    pub agent_role: String,
    pub duration_ms: Option<u64>,
    pub tokens: TokenUsage,
}
