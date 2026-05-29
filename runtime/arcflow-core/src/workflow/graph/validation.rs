//! Static validation for graph workflow definitions.

use std::collections::{HashMap, HashSet};

use crate::error::RuntimeError;
use crate::rcs::types::{GraphDefinition, StepDefinition};

/// Validates graph topology and step references before execution.
pub fn validate_graph(
    graph: &GraphDefinition,
    steps: &[StepDefinition],
) -> Result<(), RuntimeError> {
    if graph.nodes.is_empty() {
        return Err(invalid("graph must have at least one node"));
    }
    let step_ids: HashSet<_> = steps.iter().map(|s| s.id).collect();
    let mut node_ids = HashSet::new();
    for node in &graph.nodes {
        if !node_ids.insert(node.id.clone()) {
            return Err(invalid(format!("duplicate graph node id '{}'", node.id)));
        }
        if !step_ids.contains(&node.step_ref) {
            return Err(invalid(format!(
                "graph node '{}' step_ref not found in workflow steps",
                node.id
            )));
        }
    }
    if !node_ids.contains(&graph.entry_node) {
        return Err(invalid(format!(
            "entry_node '{}' not found in graph nodes",
            graph.entry_node
        )));
    }
    for edge in &graph.edges {
        if !node_ids.contains(&edge.from) {
            return Err(invalid(format!("edge from unknown node '{}'", edge.from)));
        }
        if let Some(to) = &edge.to {
            if !node_ids.contains(to) {
                return Err(invalid(format!("edge to unknown node '{to}'")));
            }
        }
    }
    for join in &graph.join_nodes {
        if !node_ids.contains(&join.id) {
            return Err(invalid(format!("join node '{}' not in graph nodes", join.id)));
        }
        for branch in &join.wait_for {
            if !node_ids.contains(branch) {
                return Err(invalid(format!(
                    "join '{}' wait_for unknown branch '{branch}'",
                    join.id
                )));
            }
        }
    }
    if graph.max_iterations == 0 {
        return Err(invalid("max_iterations must be at least 1"));
    }
    detect_unreachable_nodes(graph, &node_ids)?;
    Ok(())
}

fn detect_unreachable_nodes(
    graph: &GraphDefinition,
    node_ids: &HashSet<String>,
) -> Result<(), RuntimeError> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        if let Some(to) = edge.to.as_deref() {
            adj.entry(edge.from.as_str()).or_default().push(to);
        }
    }
    let mut visited = HashSet::new();
    let mut stack = vec![graph.entry_node.as_str()];
    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        if let Some(nexts) = adj.get(id) {
            for n in nexts {
                stack.push(n);
            }
        }
    }
    for id in node_ids {
        if !visited.contains(id.as_str()) {
            return Err(invalid(format!("unreachable graph node '{id}'")));
        }
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::InvalidWorkflowDefinition {
        reason: reason.into(),
    }
}

