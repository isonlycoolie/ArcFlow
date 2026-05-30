//! Process-wide trace store for SDK and CLI lookup (embedded mode).

use std::sync::{Mutex, OnceLock};

use super::builder::ExecutionTraceBuilder;
use super::store::{self, TraceStore};
use super::types::ExecutionTrace;

static TRACE_STORE: OnceLock<Mutex<TraceStore>> = OnceLock::new();

fn store() -> &'static Mutex<TraceStore> {
    TRACE_STORE.get_or_init(|| Mutex::new(TraceStore::new()))
}

/// Registers a hook invoked after each trace event is stored (e.g. Postgres persist).
pub fn set_trace_event_persist_hook(hook: store::PersistHook) {
    store::set_trace_event_persist_hook(hook);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceLookupError {
    StoreLockFailed,
}

/// Returns a built execution trace for `run_id`, if still retained.
pub fn get_execution_trace(run_id: &str) -> Option<ExecutionTrace> {
    try_get_execution_trace(run_id).unwrap_or_default()
}

/// Like [`get_execution_trace`] but surfaces store lock failures.
pub fn try_get_execution_trace(run_id: &str) -> Result<Option<ExecutionTrace>, TraceLookupError> {
    let guard = store()
        .lock()
        .map_err(|_| TraceLookupError::StoreLockFailed)?;
    let Some(events) = guard.get_events(run_id) else {
        return Ok(None);
    };
    let dropped = guard.events_dropped_for_run(run_id);
    Ok(Some(ExecutionTraceBuilder::build(run_id, events, dropped)))
}

/// Mutable access for workflow execution (single-threaded embedded mode).
pub fn with_store<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut TraceStore) -> R,
{
    let mut guard = store().lock().ok()?;
    Some(f(&mut guard))
}
