//! workflow.test() stub mode (Phase 2.3).

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
use arcflow_core::workflow::{ExecutionConfig, TestConfig, TestStubStep, WorkflowEngine};

fn agent(id: Uuid) -> AgentDefinition {
    AgentDefinition {
        id,
        name: "writer".into(),
        role: "author".into(),
        instructions: "stub".into(),
        tools: None,
        memory_config: None,
    }
}

#[test]
fn test_stub_overrides_step_output() {
    let wf_id = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let step_id = Uuid::new_v4();
    let workflow = WorkflowDefinition {
        id: wf_id,
        name: "test_wf".into(),
        steps: vec![StepDefinition {
            id: step_id,
            agent_id,
            order: 1,
            fallback_step_id: None,
            hitl: None,
        }],
        retry_policy: None,
        execution_mode: arcflow_core::rcs::types::ExecutionMode::Linear,
        graph: None,
    };
    let mut agents = HashMap::new();
    agents.insert(agent_id, agent(agent_id));

    let mut stubs = HashMap::new();
    stubs.insert(
        "step_1".to_string(),
        TestStubStep {
            output: Some("fixed-output".into()),
            fail_times: None,
            then_output: None,
        },
    );
    let exec_config = ExecutionConfig {
        test: Some(TestConfig {
            stub_responses: stubs,
        }),
        ..ExecutionConfig::default()
    };

    let record = WorkflowEngine::new()
        .execute_with_config(&workflow, &agents, "hello", None, None, None, 1024, 0.7, &exec_config)
        .expect("run");
    assert_eq!(record.step_outputs[0].content, "fixed-output");
}
