use std::collections::HashMap;

use tracing::{debug, error, info};
use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::{RuntimeError, StateError};
use crate::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
use crate::state::{ExecutionStepOutput, StateEngine};

use super::record::WorkflowExecutionRecord;
use super::run_error::WorkflowRunError;

struct RunLoop<'a> {
    run_id: Uuid,
    workflow_id: Uuid,
    state: &'a mut StateEngine,
    step_outputs: &'a mut Vec<ExecutionStepOutput>,
    run_input: &'a str,
}

fn partial_record(loop_ctx: &RunLoop<'_>) -> WorkflowExecutionRecord {
    WorkflowExecutionRecord {
        run_id: loop_ctx.run_id,
        workflow_id: loop_ctx.workflow_id,
        step_outputs: loop_ctx.step_outputs.clone(),
        final_state: loop_ctx.state.snapshot(),
    }
}

#[allow(clippy::result_large_err)] // partial record is intentional for halt diagnostics (ADR-002)
fn run_one_step(
    agent_runtime: &AgentRuntime,
    loop_ctx: &mut RunLoop<'_>,
    step: &StepDefinition,
    agent: &AgentDefinition,
) -> Result<(), WorkflowRunError> {
    debug!(
        run_id = %loop_ctx.run_id,
        step_id = %step.id,
        agent_id = %agent.id,
        "step execution started"
    );

    let out = match agent_runtime.execute(
        agent,
        step.id,
        &loop_ctx.state.snapshot(),
        loop_ctx.run_input,
    ) {
        Ok(output) => output,
        Err(err) => {
            error!(
                run_id = %loop_ctx.run_id,
                step_id = %step.id,
                error = %err,
                "step execution failed"
            );
            return Err(WorkflowRunError::Failed {
                error: err,
                partial: partial_record(loop_ctx),
            });
        }
    };

    loop_ctx
        .state
        .commit(out.clone())
        .map_err(|e: StateError| WorkflowRunError::Failed {
            error: RuntimeError::StateCommitFailed {
                step_id: step.id,
                reason: e.to_string(),
            },
            partial: partial_record(loop_ctx),
        })?;

    debug!(
        run_id = %loop_ctx.run_id,
        step_id = %step.id,
        "step execution completed"
    );
    loop_ctx.step_outputs.push(out);
    Ok(())
}

/// Runs validated steps in `order` sequence; halts on first agent failure with a partial record.
#[allow(clippy::result_large_err)]
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
    let mut loop_ctx = RunLoop {
        run_id,
        workflow_id: workflow.id,
        state: &mut state,
        step_outputs: &mut step_outputs,
        run_input,
    };

    for step in &steps {
        let Some(agent) = agents.get(&step.agent_id) else {
            return Err(WorkflowRunError::Aborted(RuntimeError::AgentNotFound {
                agent_id: step.agent_id,
                step_id: step.id,
            }));
        };
        run_one_step(agent_runtime, &mut loop_ctx, step, agent)?;
    }

    info!(run_id = %run_id, "workflow execution completed");
    Ok(WorkflowExecutionRecord {
        run_id,
        workflow_id: workflow.id,
        step_outputs,
        final_state: state.snapshot(),
    })
}
