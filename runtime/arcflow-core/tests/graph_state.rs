//! Graph run state round-trip (Phase 2-Pro).

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
fn graph_state_passes_observation_to_next_node() {
    let wf_id = Uuid::new_v4();
    let (a_observe, a_think) = (Uuid::new_v4(), Uuid::new_v4());
    let (s_observe, s_think) = (Uuid::new_v4(), Uuid::new_v4());

    let graph = GraphDefinition {
        entry_node: "observe".into(),
        max_iterations: 1,
        nodes: vec![
            GraphNode {
                id: "observe".into(),
                step_ref: s_observe,
                inputs: None,
                outputs: Some(vec!["observation".into()]),
            },
            GraphNode {
                id: "think".into(),
                step_ref: s_think,
                inputs: None,
                outputs: None,
            },
        ],
        edges: vec![GraphEdge {
            from: "observe".into(),
            to: Some("think".into()),
            condition: None,
        }],
        join_nodes: vec![],
    };

    let workflow = WorkflowDefinition {
        id: wf_id,
        name: "state".into(),
        steps: vec![
            StepDefinition {
                id: s_observe,
                agent_id: a_observe,
                order: 1,
                fallback_step_id: None,
                hitl: None,
            },
            StepDefinition {
                id: s_think,
                agent_id: a_think,
                order: 2,
                fallback_step_id: None,
                hitl: None,
            },
        ],
        retry_policy: None,
        execution_mode: ExecutionMode::Graph,
        graph: Some(graph),
    };

    let mut agents = HashMap::new();
    agents.insert(a_observe, agent(a_observe, "observe"));
    agents.insert(a_think, agent(a_think, "think"));

    let mut initial = serde_json::Map::new();
    initial.insert("seed".into(), serde_json::Value::String("context".into()));
    let mut exec = ExecutionConfig::default();
    exec.initial_state = Some(initial);

    let record = WorkflowEngine::new()
        .execute_with_config(
            &workflow, &agents, "task", None, None, None, 256, 0.0, &exec, None,
        )
        .unwrap();

    let think_out = record
        .step_outputs
        .iter()
        .find(|o| o.step_id == s_think)
        .expect("think step ran");
    assert!(think_out.content.contains("## Graph state"));
    assert!(think_out.content.contains("seed"));
    assert!(think_out.content.contains("observation"));
}

#[test]
fn graph_cycle_respects_max_iterations() {
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
