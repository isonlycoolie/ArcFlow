//! Parse graph workflow metadata from SDK JSON.

use arcflow_core::rcs::types::{
    ExecutionMode, GraphDefinition, GraphEdge, GraphNode, WorkflowDefinition,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct GraphPayload {
    entry_node: String,
    max_iterations: Option<u32>,
    nodes: Vec<GraphNodePayload>,
    edges: Vec<GraphEdgePayload>,
}

#[derive(Debug, Deserialize)]
struct GraphNodePayload {
    id: String,
    step_id: String,
}

#[derive(Debug, Deserialize)]
struct GraphEdgePayload {
    from: String,
    to: Option<String>,
    condition: Option<String>,
}

pub fn apply_graph_json(workflow: &mut WorkflowDefinition, raw: &str) -> Result<(), String> {
    let payload: GraphPayload =
        serde_json::from_str(raw).map_err(|e| format!("Invalid graph JSON: {e}"))?;
    let nodes: Result<Vec<GraphNode>, String> = payload
        .nodes
        .iter()
        .map(|n| {
            let step_ref = Uuid::parse_str(&n.step_id)
                .map_err(|_| format!("Invalid step_id for graph node '{}'", n.id))?;
            Ok(GraphNode {
                id: n.id.clone(),
                step_ref,
                inputs: None,
                outputs: None,
            })
        })
        .collect();
    let edges = payload
        .edges
        .iter()
        .map(|e| GraphEdge {
            from: e.from.clone(),
            to: e.to.clone(),
            condition: e.condition.clone(),
        })
        .collect();
    workflow.execution_mode = ExecutionMode::Graph;
    workflow.graph = Some(GraphDefinition {
        entry_node: payload.entry_node,
        max_iterations: payload.max_iterations.unwrap_or(100),
        nodes: nodes?,
        edges,
        join_nodes: vec![],
    });
    Ok(())
}
