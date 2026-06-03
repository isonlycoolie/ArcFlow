//! Step context chaining — step 2 sees step 1 output in assembled context.

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionMode, StepDefinition, WorkflowDefinition,
};
use arcflow_core::workflow::WorkflowEngine;

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
fn second_step_context_includes_first_step_output() {
    let (a1, a2) = (Uuid::new_v4(), Uuid::new_v4());
    let (s1, s2) = (Uuid::new_v4(), Uuid::new_v4());
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "chain".into(),
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
        external_bindings: None,
    };
    let mut agents = HashMap::new();
    agents.insert(a1, ag(a1, "researcher"));
    agents.insert(a2, ag(a2, "analyst"));
    let record = WorkflowEngine::new()
        .execute(&wf, &agents, "AAPL swing trade")
        .unwrap();
    let first = record
        .step_outputs
        .iter()
        .find(|o| o.step_id == s1)
        .unwrap();
    let second = record
        .step_outputs
        .iter()
        .find(|o| o.step_id == s2)
        .unwrap();
    assert!(second.content.contains("## Prior steps"));
    assert!(second.content.contains(&first.content));
}
