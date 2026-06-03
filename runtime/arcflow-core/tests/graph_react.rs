//! ReAct-style graph integration test (3-node cycle, iteration limit).

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionMode, GraphDefinition, GraphEdge, GraphNode, StepDefinition,
    WorkflowDefinition,
};
use arcflow_core::workflow::{ExecutionConfig, WorkflowEngine, WorkflowRunError};

fn agent(id: Uuid, role: &str) -> AgentDefinition {
    AgentDefinition {
        id,
        name: role.into(),
        role: role.into(),
        instructions: "stub".into(),
        tools: None,
        memory_config: None,
        context: None,
        tool_execution: None,
    }
}

#[test]
fn react_cycle_respects_max_iterations() {
    let wf_id = Uuid::new_v4();
    let (a_think, a_act, a_observe) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let (s_think, s_act, s_observe) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

    let graph = GraphDefinition {
        entry_node: "think".into(),
        max_iterations: 1,
        nodes: vec![
            GraphNode {
                id: "think".into(),
                step_ref: s_think,
                inputs: None,
                outputs: None,
            },
            GraphNode {
                id: "act".into(),
                step_ref: s_act,
                inputs: None,
                outputs: None,
            },
            GraphNode {
                id: "observe".into(),
                step_ref: s_observe,
                inputs: None,
                outputs: None,
            },
        ],
        edges: vec![
            GraphEdge {
                from: "think".into(),
                to: Some("act".into()),
                condition: None,
            },
            GraphEdge {
                from: "act".into(),
                to: Some("observe".into()),
                condition: None,
            },
            GraphEdge {
                from: "observe".into(),
                to: Some("think".into()),
                condition: None,
            },
        ],
        join_nodes: vec![],
    };

    let workflow = WorkflowDefinition {
        id: wf_id,
        name: "react".into(),
        steps: vec![
            StepDefinition {
                id: s_think,
                agent_id: a_think,
                order: 1,
                fallback_step_id: None,
                hitl: None,
            },
            StepDefinition {
                id: s_act,
                agent_id: a_act,
                order: 2,
                fallback_step_id: None,
                hitl: None,
            },
            StepDefinition {
                id: s_observe,
                agent_id: a_observe,
                order: 3,
                fallback_step_id: None,
                hitl: None,
            },
        ],
        retry_policy: None,
        execution_mode: ExecutionMode::Graph,
        graph: Some(graph),
    };

    let mut agents = HashMap::new();
    agents.insert(a_think, agent(a_think, "think"));
    agents.insert(a_act, agent(a_act, "act"));
    agents.insert(a_observe, agent(a_observe, "observe"));

    let err = WorkflowEngine::new()
        .execute_with_config(
            &workflow,
            &agents,
            "task",
            None,
            None,
            None,
            256,
            0.0,
            &ExecutionConfig::default(),
            None,
        )
        .unwrap_err();

    match err {
        WorkflowRunError::Aborted(_) => {}
        other => panic!("expected iteration limit abort, got {other:?}"),
    }
}
