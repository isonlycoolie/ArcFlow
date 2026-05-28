//! Sprint 5 synchronous trace emitter (ADR-008).

use crate::constants::MAX_TRACE_EVENTS_PER_RUN;
use crate::tracing::events::TraceEventKind;
use crate::tracing::store::TraceStore;
use crate::tracing::types::TraceEvent;

/// Emits structured events into a bounded [`TraceStore`].
pub struct TraceEventEmitter<'store> {
    trace_id: String,
    store: &'store mut TraceStore,
    sequence: u64,
}

impl<'store> TraceEventEmitter<'store> {
    /// Binds an emitter to a run and store for one execution.
    pub fn new(trace_id: String, store: &'store mut TraceStore) -> Self {
        Self {
            trace_id,
            store,
            sequence: 0,
        }
    }

    /// Emits an event; never panics. On capacity, records a storage warning.
    pub fn emit(&mut self, kind: TraceEventKind) {
        let sequence = self.next_sequence();
        let event = TraceEvent::new(self.trace_id.clone(), sequence, kind);
        if !self.store.append(&self.trace_id, event) {
            let dropped = self.store.events_dropped_for_run(&self.trace_id);
            let warning = TraceEvent::new(
                self.trace_id.clone(),
                self.next_sequence(),
                TraceEventKind::TraceStorageWarning {
                    run_id: self.trace_id.clone(),
                    events_dropped: dropped,
                    capacity_limit: MAX_TRACE_EVENTS_PER_RUN,
                },
            );
            let _ = self.store.append(&self.trace_id, warning);
        }
    }

    fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence += 1;
        seq
    }

    /// Run id this emitter writes under.
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }
}
