use std::collections::HashMap;

use tracing::info;
use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::{RuntimeError, StateError};
use crate::rcs::types::{AgentDefinition, WorkflowDefinition};
use crate::state::StateEngine;

use super::record::WorkflowExecutionRecord;

pub(super) fn run_sorted_steps(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    run_input: &str,
) -> Result<WorkflowExecutionRecord, RuntimeError> {
    let run_id = Uuid::new_v4();
    let mut state = StateEngine::new();
    info!(run_id = %run_id, wf = %workflow.id, "workflow started");
    let mut steps = workflow.steps.clone();
    steps.sort_by_key(|s| s.order);
    let mut step_outputs = Vec::new();
    for step in &steps {
        let agent = agents.get(&step.agent_id).ok_or(RuntimeError::AgentNotFound {
            agent_id: step.agent_id,
            step_id: step.id,
        })?;
        let out = agent_runtime.execute(agent, step.id, &state.snapshot(), run_input);
        state.commit(out.clone()).map_err(|e: StateError| {
            RuntimeError::StateCommitFailed {
                step_id: step.id,
                reason: e.to_string(),
            }
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
