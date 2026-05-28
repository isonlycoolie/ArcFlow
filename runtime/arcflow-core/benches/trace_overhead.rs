//! Trace emission overhead benchmark (Sprint 5).

use std::hint::black_box;

use arcflow_core::tracing::events::TraceEventKind;
use arcflow_core::tracing::sprint5_emitter::TraceEventEmitter;
use arcflow_core::tracing::store::TraceStore;
use arcflow_core::tracing::types::TokenUsage;
use criterion::{criterion_group, criterion_main, Criterion};

fn emit_happy_path(c: &mut Criterion) {
    c.bench_function("trace_emit_3_steps", |b| {
        b.iter(|| {
            let mut store = TraceStore::new();
            let run_id = "bench-run".to_string();
            let mut emitter = TraceEventEmitter::new(run_id.clone(), &mut store);
            emitter.emit(TraceEventKind::WorkflowStarted {
                run_id: run_id.clone(),
                workflow_name: "bench".into(),
                step_count: 3,
            });
            for idx in 0..3 {
                let step_id = format!("s{idx}");
                emitter.emit(TraceEventKind::StepStarted {
                    run_id: run_id.clone(),
                    step_id: step_id.clone(),
                    step_index: idx,
                    agent_name: "a".into(),
                    agent_role: "r".into(),
                });
                emitter.emit(TraceEventKind::StepCompleted {
                    run_id: run_id.clone(),
                    step_id,
                    step_index: idx,
                    duration_ms: 1,
                    tokens: TokenUsage::default(),
                    output_size_bytes: 0,
                });
            }
            emitter.emit(TraceEventKind::WorkflowCompleted {
                run_id,
                duration_ms: 3,
                total_tokens: TokenUsage::default(),
            });
            black_box(store.get_events("bench-run"));
        });
    });
}

criterion_group!(benches, emit_happy_path);
criterion_main!(benches);
