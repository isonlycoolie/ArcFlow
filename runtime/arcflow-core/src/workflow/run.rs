use std::collections::HashMap;

use tracing::info;
use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::{RuntimeError, StateError};
use crate::rcs::types::{AgentDefinition, WorkflowDefinition};
use crate::state::StateEngine;

use super::record::WorkflowExecutionRecord;
use super::run_error::WorkflowRunError;

/// Runs validated steps in `order` sequence; halts on first agent failure with a partial record.
pub(super) fn run_sorted_steps(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    run_input: &str,
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    let run_id = Uuid::new_v4();
    let mut state = StateEngine::new();
    info!(run_id = %run_id, wf = %workflow.id, "workflow started");
    let mut steps = workflow.steps.clone();
    steps.sort_by_key(|s| s.order);
    let mut step_outputs = Vec::new();

    for step in &steps {
        let agent = agents.get(&step.agent_id).ok_or_else(|| {
            WorkflowRunError::Aborted(RuntimeError::AgentNotFound {
                agent_id: step.agent_id,
                step_id: step.id,
            })
        })?;

        let out = agent_runtime
            .execute(agent, step.id, &state.snapshot(), run_input)
            .map_err(|error| WorkflowRunError::Failed {
                error,
                partial: WorkflowExecutionRecord {
                    run_id,
                    workflow_id: workflow.id,
                    step_outputs: step_outputs.clone(),
                    final_state: state.snapshot(),
                },
            })?;

        state
            .commit(out.clone())
            .map_err(|e: StateError| WorkflowRunError::Failed {
                error: RuntimeError::StateCommitFailed {
                    step_id: step.id,
                    reason: e.to_string(),
                },
                partial: WorkflowExecutionRecord {
                    run_id,
                    workflow_id: workflow.id,
                    step_outputs: step_outputs.clone(),
                    final_state: state.snapshot(),
                },
            })?;
        step_outputs.push(out);
    }

    info!(run_id = %run_id, "workflow completed");
    Ok(WorkflowExecutionRecord {
        run_id,
        workflow_id: workflow.id,
        step_outputs,
        final_state: state.snapshot(),
    })
}
