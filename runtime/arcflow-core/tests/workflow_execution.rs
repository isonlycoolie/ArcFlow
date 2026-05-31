//! Multi-step workflow execution and failure halt.

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::agent::STUB_FAIL_ROLE;
use arcflow_core::error::RuntimeError;
use arcflow_core::rcs::types::{AgentDefinition, ExecutionMode, StepDefinition, WorkflowDefinition};
use arcflow_core::workflow::{WorkflowEngine, WorkflowRunError};

fn ag(id: Uuid, role: &str) -> AgentDefinition {
    AgentDefinition {
        id,
        name: "n".into(),
        role: role.into(),
        instructions: "i".into(),
        tools: None,
        memory_config: None,
        context: None,
        tool_execution: None,
    }
}

#[test]
fn three_steps_with_distinct_agents_run_in_order() {
    let (a1, a2, a3) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let (s1, s2, s3) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![
            StepDefinition {
                id: s3,
                agent_id: a3,
                order: 3,
                fallback_step_id: None,
            hitl: None,
            },
            StepDefinition {
                id: s1,
                agent_id: a1,
                order: 1,
                fallback_step_id: None,
            hitl: None,
            },
            StepDefinition {
                id: s2,
                agent_id: a2,
                order: 2,
                fallback_step_id: None,
            hitl: None,
            },
        ],
        retry_policy: None,
        execution_mode: ExecutionMode::Linear,
        graph: None,
            external_bindings: None,
    };
    let mut m = HashMap::new();
    m.insert(a1, ag(a1, "A"));
    m.insert(a2, ag(a2, "B"));
    m.insert(a3, ag(a3, "C"));
    let ids: Vec<_> = WorkflowEngine::new()
        .execute(&wf, &m, "x")
        .unwrap()
        .step_outputs
        .iter()
        .map(|o| o.step_id)
        .collect();
    assert_eq!(ids, vec![s1, s2, s3]);
}

#[test]
fn workflow_halts_on_failed_step_with_partial_record() {
    let (a_ok, a_fail) = (Uuid::new_v4(), Uuid::new_v4());
    let (s1, s2, s3) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![
            StepDefinition {
                id: s1,
                agent_id: a_ok,
                order: 1,
                fallback_step_id: None,
            hitl: None,
            },
            StepDefinition {
                id: s2,
                agent_id: a_fail,
                order: 2,
                fallback_step_id: None,
            hitl: None,
            },
            StepDefinition {
                id: s3,
                agent_id: a_ok,
                order: 3,
                fallback_step_id: None,
            hitl: None,
            },
        ],
        retry_policy: None,
        execution_mode: ExecutionMode::Linear,
        graph: None,
            external_bindings: None,
    };
    let mut m = HashMap::new();
    m.insert(a_ok, ag(a_ok, "ok"));
    m.insert(a_fail, ag(a_fail, STUB_FAIL_ROLE));
    let err = WorkflowEngine::new().execute(&wf, &m, "in").unwrap_err();
    match err {
        WorkflowRunError::Failed { error, partial } => {
            assert!(matches!(
                error,
                RuntimeError::AgentExecutionFailed { step_id, .. } if step_id == s2
            ));
            assert_eq!(partial.step_outputs.len(), 1);
            assert_eq!(partial.step_outputs[0].step_id, s1);
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}
