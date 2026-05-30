//! Bounded in-process trace storage (Sprint 5 Phase 3).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::constants::{MAX_CONCURRENT_TRACES, MAX_TRACE_EVENTS_PER_RUN};
use crate::tracing::events::TraceEventKind;
use crate::tracing::types::TraceEvent;

pub type PersistHook = Arc<dyn Fn(&str, &TraceEvent) + Send + Sync>;
static PERSIST_HOOK: OnceLock<PersistHook> = OnceLock::new();

/// Registers a hook invoked after each trace event is appended (e.g. Postgres persist).
pub fn set_trace_event_persist_hook(hook: PersistHook) {
    let _ = PERSIST_HOOK.set(hook);
}

#[derive(Debug)]
struct BoundedEventBuffer {
    events: Vec<TraceEvent>,
    events_dropped: u32,
    is_complete: bool,
}

/// In-process store for workflow trace events.
#[derive(Debug, Default)]
pub struct TraceStore {
    traces: HashMap<String, BoundedEventBuffer>,
    completion_order: Vec<String>,
    total_events_dropped: u32,
}

impl TraceStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an event; returns false if the per-run cap was exceeded.
    pub fn append(&mut self, run_id: &str, event: TraceEvent) -> bool {
        let buffer = self
            .traces
            .entry(run_id.to_string())
            .or_insert_with(|| BoundedEventBuffer {
                events: Vec::new(),
                events_dropped: 0,
                is_complete: false,
            });
        if buffer.events.len() >= MAX_TRACE_EVENTS_PER_RUN as usize {
            if matches!(&event.kind, TraceEventKind::TraceStorageWarning { .. })
                && !buffer
                    .events
                    .iter()
                    .any(|stored| matches!(stored.kind, TraceEventKind::TraceStorageWarning { .. }))
            {
                buffer.events.push(event);
                return true;
            }
            buffer.events_dropped += 1;
            self.total_events_dropped += 1;
            return false;
        }
        buffer.events.push(event);
        if let Some(hook) = PERSIST_HOOK.get() {
            if let Some(last) = buffer.events.last() {
                hook(run_id, last);
            }
        }
        true
    }

    /// Events for a run in emission order.
    pub fn get_events(&self, run_id: &str) -> Option<&[TraceEvent]> {
        self.traces
            .get(run_id)
            .map(|buffer| buffer.events.as_slice())
    }

    /// Drops recorded for a single run when the per-run cap is exceeded.
    pub fn events_dropped_for_run(&self, run_id: &str) -> u32 {
        self.traces
            .get(run_id)
            .map(|buffer| buffer.events_dropped)
            .unwrap_or(0)
    }

    /// Marks a run trace complete for eviction ordering.
    pub fn mark_complete(&mut self, run_id: &str) {
        if let Some(buffer) = self.traces.get_mut(run_id) {
            buffer.is_complete = true;
            if !self.completion_order.iter().any(|id| id == run_id) {
                self.completion_order.push(run_id.to_string());
            }
        }
        self.evict_if_needed();
