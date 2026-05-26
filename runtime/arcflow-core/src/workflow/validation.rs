use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::error::RuntimeError;
use crate::rcs::types::{AgentDefinition, WorkflowDefinition};

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
    Ok(())
}
