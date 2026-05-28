//! Assembles raw events into ExecutionTrace (wired in Phase 4).

use crate::tracing::types::ExecutionTrace;

/// Builds an ExecutionTrace from stored events.
#[derive(Debug, Default)]
pub struct ExecutionTraceBuilder;

impl ExecutionTraceBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self
    }

    /// Placeholder until Phase 4 wiring.
    pub fn build_placeholder(&self) -> ExecutionTrace {
        use chrono::Utc;
        use crate::tracing::types::{ExecutionStatus, TokenUsage};

        ExecutionTrace {
            run_id: String::new(),
            workflow_name: String::new(),
            status: ExecutionStatus::Partial,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            total_tokens: TokenUsage::default(),
            steps: Vec::new(),
            events: Vec::new(),
            is_complete: false,
            events_dropped: 0,
        }
    }
}
