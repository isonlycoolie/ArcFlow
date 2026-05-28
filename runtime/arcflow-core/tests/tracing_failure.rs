//! Sprint 5 failure event ordering tests.

use std::collections::HashMap;

use arcflow_core::agent::STUB_FAIL_ROLE;
use arcflow_core::get_execution_trace;
use arcflow_core::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
use arcflow_core::tracing::events::TraceEventKind;
use arcflow_core::workflow::{WorkflowEngine, WorkflowRunError};
use uuid::Uuid;

#[test]
fn failed_step_emits_step_failed_then_workflow_failed() {
    let (a_ok, a_fail) = (Uuid::new_v4(), Uuid::new_v4());
    let (s1, s2) = (Uuid::new_v4(), Uuid::new_v4());
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "fail-wf".into(),
        steps: vec![
            StepDefinition {
                id: s1,
                agent_id: a_ok,
                order: 1,
                fallback_step_id: None,
            },
            StepDefinition {
                id: s2,
                agent_id: a_fail,
                order: 2,
                fallback_step_id: None,
            },
        ],
        retry_policy: None,
    };
    let mut agents = HashMap::new();
    agents.insert(
        a_ok,
        AgentDefinition {
            id: a_ok,
            name: "ok".into(),
            role: "ok".into(),
            instructions: "i".into(),
            tools: None,
            memory_config: None,
        },
    );
    agents.insert(
        a_fail,
        AgentDefinition {
            id: a_fail,
            name: "fail".into(),
            role: STUB_FAIL_ROLE.into(),
            instructions: "i".into(),
            tools: None,
            memory_config: None,
        },
    );

    let err = WorkflowEngine::new()
        .execute(&wf, &agents, "in")
        .unwrap_err();
    let run_id = match err {
        WorkflowRunError::Failed { partial, .. } => partial.run_id.to_string(),
        other => panic!("expected Failed, got {other:?}"),
    };

    let trace = get_execution_trace(&run_id).expect("trace retained after partial failure");
    let kinds: Vec<&str> = trace
        .events
        .iter()
        .map(|e| match &e.kind {
            TraceEventKind::StepFailed { .. } => "StepFailed",
            TraceEventKind::WorkflowFailed { .. } => "WorkflowFailed",
            TraceEventKind::StepCompleted { .. } => "StepCompleted",
            TraceEventKind::WorkflowStarted { .. } => "WorkflowStarted",
            TraceEventKind::StepStarted { .. } => "StepStarted",
            _ => "Other",
        })
        .collect();

    let step_fail = kinds.iter().rposition(|k| *k == "StepFailed");
    let wf_fail = kinds.iter().rposition(|k| *k == "WorkflowFailed");
    assert!(step_fail.is_some());
    assert!(wf_fail.is_some());
    assert_eq!(wf_fail, step_fail.map(|i| i + 1));
    assert_eq!(
        trace.status,
        arcflow_core::tracing::types::ExecutionStatus::Failed
    );
}
