//! Process-wide trace store for SDK and CLI lookup (embedded mode).

use std::sync::{Mutex, OnceLock};

use super::builder::ExecutionTraceBuilder;
use super::store::TraceStore;
use super::types::ExecutionTrace;

static TRACE_STORE: OnceLock<Mutex<TraceStore>> = OnceLock::new();

fn store() -> &'static Mutex<TraceStore> {
    TRACE_STORE.get_or_init(|| Mutex::new(TraceStore::new()))
}

/// Returns a built execution trace for `run_id`, if still retained.
pub fn get_execution_trace(run_id: &str) -> Option<ExecutionTrace> {
    let guard = store().lock().ok()?;
    let events = guard.get_events(run_id)?;
    let dropped = guard.events_dropped_for_run(run_id);
    Some(ExecutionTraceBuilder::build(run_id, events, dropped))
}

/// Mutable access for workflow execution (single-threaded embedded mode).
pub fn with_store<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut TraceStore) -> R,
{
    let mut guard = store().lock().ok()?;
    Some(f(&mut guard))
}
