//! State snapshot propagation between steps.

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::rcs::types::{AgentDefinition, ExecutionMode, StepDefinition, WorkflowDefinition};
use arcflow_core::workflow::WorkflowEngine;

fn ag(id: Uuid, role: &str) -> AgentDefinition {
    AgentDefinition {
        id,
        name: "n".into(),
        role: role.into(),
        instructions: "i".into(),
        tools: None,
        memory_config: None,
    }
}

#[test]
fn first_step_sees_empty_prior_state() {
    let a = Uuid::new_v4();
    let s1 = Uuid::new_v4();
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![StepDefinition {
            id: s1,
            agent_id: a,
            order: 1,
            fallback_step_id: None,
            hitl: None,
        }],
        retry_policy: None,
        execution_mode: ExecutionMode::Linear,
        graph: None,
    };
    let mut m = HashMap::new();
    m.insert(a, ag(a, "alpha"));
    let out = WorkflowEngine::new()
        .execute(&wf, &m, "in")
        .unwrap()
        .step_outputs[0]
        .content
        .clone();
    assert!(out.contains("prior_steps: 0"));
}

#[test]
fn second_step_sees_first_step_in_snapshot() {
    let (a1, a2) = (Uuid::new_v4(), Uuid::new_v4());
    let (s1, s2) = (Uuid::new_v4(), Uuid::new_v4());
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![
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
    };
    let mut m = HashMap::new();
    m.insert(a1, ag(a1, "first"));
    m.insert(a2, ag(a2, "second"));
    let rec = WorkflowEngine::new().execute(&wf, &m, "flow").unwrap();
    assert_eq!(rec.final_state.steps.len(), 2);
    assert_eq!(rec.final_state.steps[0].step_id, s1);
    let second = rec.step_outputs.iter().find(|o| o.step_id == s2).unwrap();
    assert!(second.content.contains("prior_steps: 1"));
}
