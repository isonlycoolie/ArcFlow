//! Event ordering contract tests (Sprint 5).

use arcflow_core::tracing::events::TraceEventKind;
use arcflow_core::tracing::sprint5_emitter::TraceEventEmitter;
use arcflow_core::tracing::store::TraceStore;
use arcflow_core::tracing::types::TokenUsage;

fn emit_three_step_happy_path(emitter: &mut TraceEventEmitter<'_>) {
    let run = emitter.trace_id().to_string();
    emitter.emit(TraceEventKind::WorkflowStarted {
        run_id: run.clone(),
        workflow_name: "ordering".into(),
        step_count: 3,
    });
    for (idx, name) in ["a", "b", "c"].iter().enumerate() {
        let step_id = format!("step-{idx}");
        emitter.emit(TraceEventKind::StepStarted {
            run_id: run.clone(),
            step_id: step_id.clone(),
            step_index: idx,
            agent_name: (*name).into(),
            agent_role: "test".into(),
        });
        emitter.emit(TraceEventKind::StepCompleted {
            run_id: run.clone(),
            step_id,
            step_index: idx,
            duration_ms: 1,
            tokens: TokenUsage::default(),
            output_size_bytes: 0,
        });
    }
    emitter.emit(TraceEventKind::WorkflowCompleted {
        run_id: run,
        duration_ms: 3,
        total_tokens: TokenUsage::default(),
    });
}

#[test]
fn test_workflow_engine_emits_started_then_steps_then_completed() {
    let mut store = TraceStore::new();
    let run_id = "run-order".to_string();
    let mut emitter = TraceEventEmitter::new(run_id.clone(), &mut store);
    emit_three_step_happy_path(&mut emitter);

    let events = store.get_events(&run_id).expect("events");
    let kinds: Vec<String> = events
        .iter()
        .map(|e| match &e.kind {
            TraceEventKind::WorkflowStarted { .. } => "WorkflowStarted",
            TraceEventKind::StepStarted { .. } => "StepStarted",
            TraceEventKind::StepCompleted { .. } => "StepCompleted",
            TraceEventKind::WorkflowCompleted { .. } => "WorkflowCompleted",
            other => panic!("unexpected kind: {other:?}"),
        })
        .map(str::to_string)
        .collect();

    assert_eq!(kinds.first().map(String::as_str), Some("WorkflowStarted"));
    assert_eq!(kinds.last().map(String::as_str), Some("WorkflowCompleted"));
    assert_eq!(kinds.iter().filter(|k| *k == "StepStarted").count(), 3);
    assert_eq!(kinds.iter().filter(|k| *k == "StepCompleted").count(), 3);
}
