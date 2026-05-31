//! Resolve and validate external bindings on a workflow.

use uuid::Uuid;

use crate::error::RuntimeError;
use crate::rcs::types::{ExternalBinding, WorkflowDefinition};

/// Returns the binding with `binding_id` if declared on the workflow.
pub fn find_binding<'a>(
    workflow: &'a WorkflowDefinition,
    binding_id: &str,
) -> Option<&'a ExternalBinding> {
    workflow.external_bindings.as_ref()?.iter().find(|b| b.id == binding_id)
}

/// Validates binding step references and unique ids.
pub fn validate_bindings(workflow: &WorkflowDefinition) -> Result<(), RuntimeError> {
    let Some(bindings) = &workflow.external_bindings else {
        return Ok(());
    };
    let step_ids: std::collections::HashSet<Uuid> = workflow.steps.iter().map(|s| s.id).collect();
    let mut seen = std::collections::HashSet::new();
    for b in bindings {
        if b.id.trim().is_empty() {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: "external binding id must be non-empty".into(),
            });
        }
        if !seen.insert(b.id.clone()) {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: format!("duplicate external binding id '{}'", b.id),
            });
        }
        if !step_ids.contains(&b.attach_to_step_id) {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: format!(
                    "external binding '{}' attach_to_step_id not found in workflow steps",
                    b.id
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rcs::types::{
        ExternalBindingKind, ExternalBindingMode, StepDefinition, WorkflowDefinition,
    };
    use serde_json::json;
    use uuid::Uuid;

    fn sample_workflow(step_id: Uuid) -> WorkflowDefinition {
        WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "w".into(),
            steps: vec![StepDefinition {
                id: step_id,
                agent_id: Uuid::new_v4(),
                order: 0,
                fallback_step_id: None,
                hitl: None,
            }],
            retry_policy: None,
            execution_mode: crate::rcs::types::ExecutionMode::Linear,
            graph: None,
            external_bindings: None,
        }
    }

    #[test]
    fn find_binding_returns_match() {
        let step_id = Uuid::new_v4();
        let mut wf = sample_workflow(step_id);
        wf.external_bindings = Some(vec![ExternalBinding {
            id: "b1".into(),
            kind: ExternalBindingKind::BrowserAutomation,
            attach_to_step_id: step_id,
            mode: ExternalBindingMode::AsyncCallback,
            outcome_schema: json!({"type": "object", "required": ["status"]}),
            recovery: None,
        }]);
        assert!(find_binding(&wf, "b1").is_some());
        assert!(find_binding(&wf, "missing").is_none());
    }

    #[test]
    fn validate_rejects_unknown_step() {
        let wf = WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "w".into(),
            steps: vec![],
            retry_policy: None,
            execution_mode: crate::rcs::types::ExecutionMode::Linear,
            graph: None,
            external_bindings: Some(vec![ExternalBinding {
                id: "b1".into(),
                kind: ExternalBindingKind::Custom,
                attach_to_step_id: Uuid::new_v4(),
                mode: ExternalBindingMode::SyncTool,
                outcome_schema: json!({}),
                recovery: None,
            }]),
        };
        assert!(validate_bindings(&wf).is_err());
    }
}
