use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tracing::{debug, error, info};
use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::{RuntimeError, StateError};
use crate::memory::MemoryCoordinator;
use crate::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
use crate::state::{ExecutionStepOutput, StateEngine};
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::tracing::{
    emitter::TraceEmitter, events::TraceEventKind, sprint5_emitter::TraceEventEmitter, with_store,
    TokenUsage,
};

use super::context::ExecutionContext;
use super::record::WorkflowExecutionRecord;
use super::run_error::WorkflowRunError;

struct RunLoop<'a> {
    run_id: Uuid,
    workflow_id: Uuid,
    state: &'a mut StateEngine,
    step_outputs: &'a mut Vec<ExecutionStepOutput>,
    run_input: &'a str,
}

fn partial_record(loop_ctx: &RunLoop<'_>, legacy: &TraceEmitter) -> WorkflowExecutionRecord {
    WorkflowExecutionRecord {
        run_id: loop_ctx.run_id,
        workflow_id: loop_ctx.workflow_id,
        step_outputs: loop_ctx.step_outputs.clone(),
        final_state: loop_ctx.state.snapshot(),
        trace_events: legacy.events().to_vec(),
    }
}

#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
fn run_one_step(
    agent_runtime: &AgentRuntime,
    loop_ctx: &mut RunLoop<'_>,
    step: &StepDefinition,
    step_index: usize,
    agent: &AgentDefinition,
    memory: &mut MemoryCoordinator,
    legacy: &mut TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_key: &str,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    workflow_started: Instant,
) -> Result<(), WorkflowRunError> {
    debug!(
        run_id = %loop_ctx.run_id,
        step_id = %step.id,
        agent_id = %agent.id,
        "step execution started"
    );

    sprint5.emit(TraceEventKind::StepStarted {
        run_id: run_key.to_string(),
        step_id: step.id.to_string(),
        step_index,
        agent_name: agent.name.clone(),
        agent_role: agent.role.clone(),
    });

    let step_started = Instant::now();
    let out = {
        let mut exec_ctx = ExecutionContext {
            tool_runtime,
            tool_invoker,
            memory,
            legacy,
            sprint5,
            run_id: run_key.to_string(),
        };
        agent_runtime.execute_with_context(
            agent,
            step.id,
            &loop_ctx.state.snapshot(),
            loop_ctx.run_input,
            Some(&mut exec_ctx),
        )
    };

    let out = match out {
        Ok(output) => output,
        Err(err) => {
            error!(
                run_id = %loop_ctx.run_id,
                step_id = %step.id,
                error = %err,
                "step execution failed"
            );
            sprint5.emit(TraceEventKind::StepFailed {
                run_id: run_key.to_string(),
                step_id: step.id.to_string(),
                step_index,
                duration_ms: step_started.elapsed().as_millis() as u64,
                error_code: "step_failed".into(),
                error_message: err.to_string(),
            });
            sprint5.emit(TraceEventKind::WorkflowFailed {
                run_id: run_key.to_string(),
                duration_ms: workflow_started.elapsed().as_millis() as u64,
                failed_step_index: Some(step_index),
                error_code: "step_failed".into(),
            });
            return Err(WorkflowRunError::Failed {
                error: err,
                partial: partial_record(loop_ctx, legacy),
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
            partial: partial_record(loop_ctx, legacy),
        })?;

    sprint5.emit(TraceEventKind::StepCompleted {
        run_id: run_key.to_string(),
        step_id: step.id.to_string(),
        step_index,
        duration_ms: step_started.elapsed().as_millis() as u64,
        tokens: TokenUsage::default(),
        output_size_bytes: out.content.len(),
    });

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
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    let run_id = Uuid::new_v4();
    let run_key = run_id.to_string();
    let workflow_started = Instant::now();

    with_store(|store| {
        let mut steps = workflow.steps.clone();
        steps.sort_by_key(|s| s.order);
        let step_count = steps.len();

        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), store);
        let mut legacy = TraceEmitter::new(run_id);

        sprint5.emit(TraceEventKind::WorkflowStarted {
            run_id: run_key.clone(),
            workflow_name: workflow.name.clone(),
            step_count,
        });
        legacy.workflow_started();
        info!(run_id = %run_id, workflow_id = %workflow.id, "workflow execution started");

        let mut state = StateEngine::new();
        let mut memory = MemoryCoordinator::new(run_id);
        let mut step_outputs = Vec::new();
        let mut loop_ctx = RunLoop {
            run_id,
            workflow_id: workflow.id,
            state: &mut state,
            step_outputs: &mut step_outputs,
            run_input,
        };

        for (step_index, step) in steps.iter().enumerate() {
            let Some(agent) = agents.get(&step.agent_id) else {
                return Err(WorkflowRunError::Aborted(RuntimeError::AgentNotFound {
                    agent_id: step.agent_id,
                    step_id: step.id,
                }));
            };
            run_one_step(
                agent_runtime,
                &mut loop_ctx,
                step,
                step_index,
                agent,
                &mut memory,
                &mut legacy,
                &mut sprint5,
                &run_key,
                tool_runtime,
                tool_invoker.clone(),
                workflow_started,
            )?;
        }

        let duration_ms = workflow_started.elapsed().as_millis() as u64;
        sprint5.emit(TraceEventKind::WorkflowCompleted {
            run_id: run_key.clone(),
            duration_ms,
            total_tokens: TokenUsage::default(),
        });
        legacy.workflow_completed();
        info!(run_id = %run_id, "workflow execution completed");
        let trace_events = legacy.events().to_vec();
        drop(sprint5);

        store.mark_complete(&run_key);
        Ok(WorkflowExecutionRecord {
            run_id,
            workflow_id: workflow.id,
            step_outputs,
            final_state: state.snapshot(),
            trace_events,
        })
    })
    .unwrap_or_else(|| {
        Err(WorkflowRunError::Aborted(RuntimeError::StateCommitFailed {
            step_id: Uuid::nil(),
            reason: "trace store lock unavailable".into(),
        }))
    })
}
