//! TraceStore bounded retention tests (Sprint 5).

use arcflow_core::constants::MAX_TRACE_EVENTS_PER_RUN;
use arcflow_core::tracing::events::TraceEventKind;
use arcflow_core::tracing::store::TraceStore;
use arcflow_core::tracing::types::TraceEvent;

fn sample_event(run_id: &str, seq: u64) -> TraceEvent {
    TraceEvent::new(
        run_id.to_string(),
        seq,
        TraceEventKind::WorkflowStarted {
            run_id: run_id.to_string(),
            workflow_name: "test".into(),
            step_count: 1,
        },
    )
}

#[test]
fn test_store_append_respects_per_run_capacity() {
    let mut store = TraceStore::new();
    let run_id = "run-capacity";
    let limit = MAX_TRACE_EVENTS_PER_RUN as usize;

    for seq in 0..limit {
        assert!(store.append(run_id, sample_event(run_id, seq as u64)));
    }
    assert!(!store.append(run_id, sample_event(run_id, limit as u64)));
    assert_eq!(store.events_dropped_for_run(run_id), 1);
    assert_eq!(store.get_events(run_id).map(|e| e.len()), Some(limit));
}

#[test]
fn test_store_get_events_returns_none_for_unknown_run() {
    let store = TraceStore::new();
    assert!(store.get_events("missing").is_none());
}
