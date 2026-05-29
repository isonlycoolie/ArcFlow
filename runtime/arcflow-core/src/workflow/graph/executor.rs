//! Graph edge resolution and iteration guards.

use std::collections::HashMap;

use crate::error::RuntimeError;
use crate::rcs::types::GraphDefinition;

/// Resolves next graph node ids after a node completes.
pub struct GraphExecutor {
    graph: GraphDefinition,
    node_visits: HashMap<String, u32>,
    allow_parallel: bool,
}

impl GraphExecutor {
    pub fn new(graph: GraphDefinition) -> Self {
        Self {
            graph,
            node_visits: HashMap::new(),
            allow_parallel: false,
        }
    }

    pub fn with_parallel(mut self) -> Self {
        self.allow_parallel = true;
        self
    }

    pub fn entry_node(&self) -> &str {
        &self.graph.entry_node
    }

    pub fn max_iterations(&self) -> u32 {
        self.graph.max_iterations
    }

    pub fn total_visits(&self) -> u32 {
        self.node_visits.values().sum()
    }

    /// Records a visit before executing a node; fails when limit exceeded.
    pub fn record_visit(&mut self, node_id: &str) -> Result<(), RuntimeError> {
        let count = self.node_visits.entry(node_id.to_string()).or_insert(0);
        *count += 1;
        if *count > self.graph.max_iterations {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: format!(
                    "graph iteration limit {} exceeded at node '{node_id}'",
                    self.graph.max_iterations
                ),
            });
        }
        Ok(())
    }

    /// Returns target node ids for the edge key (None = unconditional edges only).
    pub fn resolve_next(
        &self,
        completed_node: &str,
        edge_key: Option<&str>,
    ) -> Result<Vec<String>, RuntimeError> {
        let mut targets = Vec::new();
        for edge in &self.graph.edges {
            if edge.from != completed_node {
                continue;
            }
            let Some(to) = edge.to.as_ref() else {
                continue;
            };
            match &edge.condition {
                None => targets.push(to.clone()),
                Some(cond) if edge_key == Some(cond.as_str()) => targets.push(to.clone()),
                Some(_) => {}
            }
        }
        if targets.is_empty() {
            return Ok(vec![]);
        }
        if !self.allow_parallel && targets.len() > 1 {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: format!(
                    "multiple graph edges from '{completed_node}' require parallel mode"
                ),
            });
        }
        Ok(targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::rcs::types::{GraphEdge, GraphNode};

    fn sample_graph() -> GraphDefinition {
        GraphDefinition {
            entry_node: "a".into(),
            max_iterations: 5,
