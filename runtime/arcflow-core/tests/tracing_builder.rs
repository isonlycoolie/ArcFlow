//! ExecutionTraceBuilder tests (Sprint 5).

use arcflow_core::tracing::builder::ExecutionTraceBuilder;
use arcflow_core::tracing::events::TraceEventKind;
use arcflow_core::tracing::types::{ExecutionStatus, TokenUsage, TraceEvent};

#[test]
fn test_builder_assembles_three_step_completed_trace() {
    let run_id = "run-build";
    let mut events = vec![TraceEvent::new(
        run_id.into(),
        0,
        TraceEventKind::WorkflowStarted {
            run_id: run_id.into(),
            workflow_name: "wf".into(),
            step_count: 3,
        },
    )];
    let mut seq = 1_u64;
    for (idx, sid) in ["s0", "s1", "s2"].iter().enumerate() {
        events.push(TraceEvent::new(
            run_id.into(),
            seq,
            TraceEventKind::StepStarted {
                run_id: run_id.into(),
                step_id: (*sid).into(),
                step_index: idx,
                agent_name: "agent".into(),
                agent_role: "role".into(),
            },
        ));
        seq += 1;
        events.push(TraceEvent::new(
            run_id.into(),
            seq,
            TraceEventKind::StepCompleted {
                run_id: run_id.into(),
                step_id: (*sid).into(),
                step_index: idx,
                duration_ms: 1,
                tokens: TokenUsage {
                    prompt_tokens: 1,
                    completion_tokens: 2,
                    total_tokens: 3,
                },
                output_size_bytes: 0,
            },
        ));
        seq += 1;
    }
    events.push(TraceEvent::new(
        run_id.into(),
        seq,
        TraceEventKind::WorkflowCompleted {
            run_id: run_id.into(),
            duration_ms: 9,
            total_tokens: TokenUsage {
                prompt_tokens: 3,
                completion_tokens: 6,
                total_tokens: 9,
            },
        },
    ));

    let trace = ExecutionTraceBuilder::build(run_id, &events, 0);
    assert_eq!(trace.status, ExecutionStatus::Completed);
    assert_eq!(trace.steps.len(), 3);
    assert_eq!(trace.total_tokens.total_tokens, 9);
}
