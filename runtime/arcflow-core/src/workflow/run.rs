use std::collections::HashMap;

use tracing::{debug, error, info};
use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::{RuntimeError, StateError};
use crate::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
use crate::state::{ExecutionStepOutput, StateEngine};

use super::record::WorkflowExecutionRecord;
use super::run_error::WorkflowRunError;

fn partial_record(
    run_id: Uuid,
    workflow_id: Uuid,
    step_outputs: &[ExecutionStepOutput],
    state: &StateEngine,
) -> WorkflowExecutionRecord {
    WorkflowExecutionRecord {
        run_id,
        workflow_id,
        step_outputs: step_outputs.to_vec(),
        final_state: state.snapshot(),
    }
}

fn run_one_step(
    agent_runtime: &AgentRuntime,
    run_id: Uuid,
    workflow_id: Uuid,
    step: &StepDefinition,
    agent: &AgentDefinition,
    state: &mut StateEngine,
    step_outputs: &mut Vec<ExecutionStepOutput>,
    run_input: &str,
) -> Result<(), WorkflowRunError> {
    debug!(
        run_id = %run_id,
        step_id = %step.id,
        agent_id = %agent.id,
        "step execution started"
    );

    let out = match agent_runtime.execute(agent, step.id, &state.snapshot(), run_input) {
        Ok(output) => output,
        Err(err) => {
            error!(
                run_id = %run_id,
                step_id = %step.id,
                error = %err,
                "step execution failed"
            );
            return Err(WorkflowRunError::Failed {
                error: err,
                partial: partial_record(run_id, workflow_id, step_outputs, state),
            });
        }
    };

    state
        .commit(out.clone())
        .map_err(|e: StateError| WorkflowRunError::Failed {
            error: RuntimeError::StateCommitFailed {
                step_id: step.id,
                reason: e.to_string(),
            },
            partial: partial_record(run_id, workflow_id, step_outputs, state),
        })?;

    debug!(run_id = %run_id, step_id = %step.id, "step execution completed");
    step_outputs.push(out);
    Ok(())
}

/// Runs validated steps in `order` sequence; halts on first agent failure with a partial record.
pub(super) fn run_sorted_steps(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    run_input: &str,
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    let run_id = Uuid::new_v4();
    let mut state = StateEngine::new();
    info!(run_id = %run_id, workflow_id = %workflow.id, "workflow execution started");
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
        run_one_step(
            agent_runtime,
            run_id,
            workflow.id,
            step,
            agent,
            &mut state,
            &mut step_outputs,
            run_input,
        )?;
    }

    info!(run_id = %run_id, "workflow execution completed");
    Ok(WorkflowExecutionRecord {
        run_id,
        workflow_id: workflow.id,
        step_outputs,
        final_state: state.snapshot(),
    })
}
