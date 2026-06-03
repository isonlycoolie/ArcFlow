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

    #[allow(dead_code)]
    pub fn entry_node(&self) -> &str {
        &self.graph.entry_node
    }

    #[allow(dead_code)]
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
        #[cfg(feature = "otel")]
        crate::tracing::otel_metrics::record_graph_iteration(node_id);
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
            nodes: vec![
                GraphNode {
                    id: "a".into(),
                    step_ref: Uuid::new_v4(),
                    inputs: None,
                    outputs: None,
                },
                GraphNode {
                    id: "b".into(),
                    step_ref: Uuid::new_v4(),
                    inputs: None,
                    outputs: None,
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "a".into(),
                    to: Some("b".into()),
                    condition: Some("go".into()),
                },
                GraphEdge {
                    from: "b".into(),
                    to: None,
                    condition: None,
                },
            ],
            join_nodes: vec![],
        }
    }

    #[test]
    fn conditional_edge_routes_by_key() {
        let ex = GraphExecutor::new(sample_graph());
        let next = ex.resolve_next("a", Some("go")).expect("resolve");
        assert_eq!(next, vec!["b".to_string()]);
    }

    #[test]
    fn terminal_node_returns_empty() {
        let ex = GraphExecutor::new(sample_graph());
        let next = ex.resolve_next("b", None).expect("resolve");
        assert!(next.is_empty());
    }

    #[test]
    fn visit_limit_enforced() {
        let mut ex = GraphExecutor::new(sample_graph());
        for _ in 0..5 {
            ex.record_visit("a").expect("ok");
        }
        assert!(ex.record_visit("a").is_err());
    }

    #[test]
    fn parallel_edges_allowed_with_flag() {
        let mut graph = sample_graph();
        graph.edges.push(GraphEdge {
            from: "a".into(),
            to: Some("c".into()),
            condition: Some("alt".into()),
        });
        graph.nodes.push(GraphNode {
            id: "c".into(),
            step_ref: Uuid::new_v4(),
            inputs: None,
            outputs: None,
        });
        let ex = GraphExecutor::new(graph).with_parallel();
        let next = ex.resolve_next("a", Some("go")).expect("resolve");
        assert_eq!(next.len(), 1);
    }
}
