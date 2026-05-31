use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::error::RuntimeError;
use crate::rcs::types::{AgentDefinition, ExecutionMode, WorkflowDefinition};

use super::graph::validate_graph;

/// Static checks before any step runs.
pub(crate) fn validate_workflow(
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
) -> Result<(), RuntimeError> {
    if workflow.name.trim().is_empty() {
        return Err(RuntimeError::InvalidWorkflowDefinition {
            reason: "workflow name must be non-empty".into(),
        });
    }
    if workflow.steps.is_empty() {
        return Err(RuntimeError::InvalidWorkflowDefinition {
            reason: "workflow must have at least one step".into(),
        });
    }
    let mut seen = HashSet::new();
    for step in &workflow.steps {
        if !seen.insert(step.id) {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: "duplicate step id in workflow definition".into(),
            });
        }
        if !agents.contains_key(&step.agent_id) {
            return Err(RuntimeError::AgentNotFound {
                agent_id: step.agent_id,
                step_id: step.id,
            });
        }
    }
    match workflow.execution_mode {
        ExecutionMode::Linear => {
            if workflow.graph.is_some() {
                return Err(RuntimeError::InvalidWorkflowDefinition {
                    reason: "graph block not allowed in linear execution_mode".into(),
                });
            }
        }
        ExecutionMode::Graph => {
            let Some(graph) = &workflow.graph else {
                return Err(RuntimeError::InvalidWorkflowDefinition {
                    reason: "graph block required when execution_mode is graph".into(),
                });
            };
            validate_graph(graph, &workflow.steps)?;
        }
    }
    crate::external::validate_bindings(workflow)?;
    Ok(())
}
