//! Deterministic stub execution across runs.

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionMode, StepDefinition, WorkflowDefinition,
};
use arcflow_core::workflow::{WorkflowEngine, WorkflowExecutionRecord};

#[test]
fn identical_inputs_produce_identical_step_contents() {
    let a = Uuid::new_v4();
    let (s1, s2) = (Uuid::new_v4(), Uuid::new_v4());
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![
            StepDefinition {
                id: s1,
                agent_id: a,
                order: 1,
                fallback_step_id: None,
                hitl: None,
            },
            StepDefinition {
                id: s2,
                agent_id: a,
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
    m.insert(
        a,
        AgentDefinition {
            id: a,
            name: "n".into(),
            role: "r".into(),
            instructions: "i".into(),
            tools: None,
            memory_config: None,
            context: None,
            tool_execution: None,
        },
    );
    let e = WorkflowEngine::new();
    let contents = |run: &WorkflowExecutionRecord| {
        run.step_outputs
            .iter()
            .map(|o| o.content.clone())
            .collect::<Vec<_>>()
    };
    let r1 = e.execute(&wf, &m, "same").unwrap();
    let r2 = e.execute(&wf, &m, "same").unwrap();
    assert_eq!(contents(&r1), contents(&r2));
    assert_ne!(r1.run_id, r2.run_id);
}
