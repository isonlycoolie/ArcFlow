//! Minimal trace emission for Sprint 4 (tools + memory).

use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::rcs::types::{TraceEvent, TraceEventKind};

/// Collects RCS trace events for a single workflow run (no payload values).
#[derive(Debug, Default)]
pub struct TraceEmitter {
    trace_id: Uuid,
    events: Vec<TraceEvent>,
}

impl TraceEmitter {
    /// Starts a new emitter correlated by `trace_id`.
    pub fn new(trace_id: Uuid) -> Self {
        Self {
            trace_id,
            events: Vec::new(),
        }
    }

    /// Immutable view of emitted events.
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    fn push(&mut self, kind: TraceEventKind, step_id: Option<Uuid>, data: Option<Value>) {
        self.events.push(TraceEvent {
            trace_id: self.trace_id,
            event_kind: kind,
            timestamp: Utc::now(),
            step_id,
            data,
        });
    }

    /// Records workflow start.
    pub fn workflow_started(&mut self) {
        self.push(TraceEventKind::WorkflowStarted, None, None);
    }

    /// Records workflow completion.
    pub fn workflow_completed(&mut self) {
        self.push(TraceEventKind::WorkflowCompleted, None, None);
    }

    /// Records a tool invocation (metadata only).
    pub fn tool_executed(
        &mut self,
        step_id: Option<Uuid>,
        tool_name: &str,
        status: &str,
        duration_ms: u64,
    ) {
        self.push(
            TraceEventKind::ToolExecuted,
            step_id,
            Some(json!({
                "tool_name": tool_name,
                "status": status,
                "duration_ms": duration_ms,
            })),
        );
    }

    /// Records a memory read (key metadata only).
    pub fn memory_read(&mut self, step_id: Option<Uuid>, memory_type: &str, key_len: usize) {
        self.push(
            TraceEventKind::MemoryRead,
            step_id,
            Some(json!({
                "memory_type": memory_type,
                "key_len": key_len,
            })),
        );
    }

    /// Records a memory write (key metadata only).
    pub fn memory_write(&mut self, step_id: Option<Uuid>, memory_type: &str, value_len: usize) {
        self.push(
            TraceEventKind::MemoryWrite,
            step_id,
            Some(json!({
                "memory_type": memory_type,
                "value_len": value_len,
            })),
        );
    }
}
