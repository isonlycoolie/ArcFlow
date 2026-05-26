//! Integration tests for the Sprint 2 workflow stack.

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
use arcflow_core::workflow::WorkflowEngine;

fn ag(id: Uuid) -> AgentDefinition {
    AgentDefinition {
        id,
        name: "n".into(),
        role: "r".into(),
        instructions: "i".into(),
        tools: None,
        memory_config: None,
    }
}

#[test]
fn one_step_produces_one_output() {
    let a = Uuid::new_v4();
    let sid = Uuid::new_v4();
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![StepDefinition {
            id: sid,
            agent_id: a,
            order: 1,
            fallback_step_id: None,
        }],
        retry_policy: None,
    };
    let mut m = HashMap::new();
    m.insert(a, ag(a));
    let r = WorkflowEngine::new().execute(&wf, &m, "hello").unwrap();
    assert_eq!(r.step_outputs.len(), 1);
    assert_eq!(r.step_outputs[0].step_id, sid);
}

#[test]
fn execution_order_sorts_by_step_order_field() {
    let a = Uuid::new_v4();
    let (s1, s2, s3) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![
            StepDefinition {
                id: s3,
                agent_id: a,
                order: 3,
                fallback_step_id: None,
            },
            StepDefinition {
                id: s1,
                agent_id: a,
                order: 1,
                fallback_step_id: None,
            },
            StepDefinition {
                id: s2,
                agent_id: a,
                order: 2,
                fallback_step_id: None,
            },
        ],
        retry_policy: None,
    };
    let mut m = HashMap::new();
    m.insert(a, ag(a));
    let got: Vec<_> = WorkflowEngine::new()
        .execute(&wf, &m, "x")
        .unwrap()
        .step_outputs
        .iter()
        .map(|o| o.step_id)
        .collect();
    assert_eq!(got, vec![s1, s2, s3]);
    let e = WorkflowEngine::new();
    let c1 = e.execute(&wf, &m, "z").unwrap().step_outputs[0].content.clone();
    let c2 = e.execute(&wf, &m, "z").unwrap().step_outputs[0].content.clone();
    assert_eq!(c1, c2);
}
