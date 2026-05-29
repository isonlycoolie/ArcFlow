use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tracing::{debug, error, info};
use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::{RuntimeError, StateError};
use crate::human::{interrupt_for_human, ApprovalResult};
use crate::memory::MemoryCoordinator;
use crate::providers::ModelProvider;
use crate::rcs::types::{AgentDefinition, ExecutionStatus, StepDefinition, WorkflowDefinition};
use crate::state::{ExecutionStepOutput, StateEngine};
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::tracing::{
    emitter::TraceEmitter, events::TraceEventKind, otel_export::maybe_export_trace,
    sprint5_emitter::TraceEventEmitter, with_store, TokenUsage,
};

use super::context::ExecutionContext;
use super::execution_config::ExecutionConfig;
use super::record::WorkflowExecutionRecord;
use super::run_error::WorkflowRunError;
use crate::retry::RetryConfig;

/// Parameters when resuming from PostgreSQL recovery state.
pub struct ResumeParams {
    pub original_run_id: Uuid,
    pub recovery_id: String,
    pub precompleted: Vec<ExecutionStepOutput>,
    pub start_step_index: usize,
    pub run_input: String,
    /// Injected human approval when resuming from HITL interrupt.
    pub approval: Option<ApprovalResult>,
}

pub(crate) struct RunLoop<'a> {
    pub(crate) run_id: Uuid,
    pub(crate) workflow_id: Uuid,
    pub(crate) state: &'a mut StateEngine,
    pub(crate) step_outputs: &'a mut Vec<ExecutionStepOutput>,
    pub(crate) run_input: &'a str,
}

pub(crate) fn check_workflow_timeout(
    workflow_timeout: Option<std::time::Duration>,
    workflow_started: Instant,
    run_key: &str,
    sprint5: &mut TraceEventEmitter<'_>,
) -> Result<(), RuntimeError> {
    let Some(limit) = workflow_timeout else {
        return Ok(());
    };
    let elapsed = workflow_started.elapsed();
    if elapsed <= limit {
        return Ok(());
    }
    let configured_ms = limit.as_millis() as u64;
    let elapsed_ms = elapsed.as_millis() as u64;
    sprint5.emit(TraceEventKind::TimeoutEnforced {
        run_id: run_key.to_string(),
        step_id: String::new(),
        timeout_type: "workflow".to_string(),
        configured_ms,
        elapsed_ms,
    });
    Err(RuntimeError::WorkflowTimeout {
        configured_ms,
        elapsed_ms,
    })
}

#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
fn fail_step(
    loop_ctx: &RunLoop<'_>,
    legacy: &TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_key: &str,
    step: &StepDefinition,
    step_index: usize,
    workflow_started: Instant,
    step_started: Instant,
    recovery_enabled: bool,
    err: RuntimeError,
) -> Result<(), WorkflowRunError> {
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
    let partial = partial_record(loop_ctx, legacy);
    crate::recovery::persist::persist_if_enabled(
        recovery_enabled,
        loop_ctx.workflow_id,
        loop_ctx.run_id,
        loop_ctx.run_input,
        &partial.step_outputs,
        step_index,
        "step_failed",
    );
    Err(WorkflowRunError::Failed {
        error: err,
        partial,
    })
}

pub(crate) fn partial_record(loop_ctx: &RunLoop<'_>, legacy: &TraceEmitter) -> WorkflowExecutionRecord {
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
fn execute_agent_for_step(
    agent_runtime: &AgentRuntime,
    agent: &AgentDefinition,
    step: &StepDefinition,
    loop_ctx: &RunLoop<'_>,
    memory: &mut MemoryCoordinator,
    legacy: &mut TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_key: &str,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    provider: Option<Arc<dyn ModelProvider>>,
    provider_max_tokens: u32,
    provider_temperature: f32,
    retry_config: Option<RetryConfig>,
    step_timeout: Option<std::time::Duration>,
    workflow_deadline: Option<Instant>,
) -> Result<ExecutionStepOutput, RuntimeError> {
    let state_snapshot = loop_ctx.state.snapshot();
    let mut exec_ctx = ExecutionContext {
        tool_runtime,
        tool_invoker,
        memory,
        legacy,
        sprint5,
        run_id: run_key.to_string(),
        provider,
        provider_max_tokens,
        provider_temperature,
        retry_config,
        step_timeout,
        workflow_deadline,
    };
    agent_runtime.execute_with_context(
        agent,
        step.id,
        &state_snapshot,
        loop_ctx.run_input,
        Some(&mut exec_ctx),
    )
}

#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_one_step(
    agent_runtime: &AgentRuntime,
    loop_ctx: &mut RunLoop<'_>,
    step: &StepDefinition,
    step_index: usize,
    agent: &AgentDefinition,
    all_steps: &[StepDefinition],
    agents: &HashMap<Uuid, AgentDefinition>,
    memory: &mut MemoryCoordinator,
    legacy: &mut TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_key: &str,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    workflow_started: Instant,
    provider: Option<Arc<dyn ModelProvider>>,
    provider_max_tokens: u32,
    provider_temperature: f32,
    retry_config: Option<RetryConfig>,
    step_timeout: Option<std::time::Duration>,
    workflow_deadline: Option<Instant>,
    recovery_enabled: bool,
    approval: Option<&ApprovalResult>,
) -> Result<(), WorkflowRunError> {
    debug!(
        run_id = %loop_ctx.run_id,
        step_id = %step.id,
        agent_id = %agent.id,
        "step execution started"
    );

    if let Some(ref hitl) = step.hitl {
        if let Some(result) = approval {
            if !result.approved {
                return fail_step(
                    loop_ctx,
                    legacy,
                    sprint5,
                    run_key,
                    step,
                    step_index,
                    workflow_started,
                    Instant::now(),
                    recovery_enabled,
                    RuntimeError::HumanRejected {
                        approval_key: hitl.approval_key.clone(),
                    },
                );
            }
            let content = serde_json::json!({
                "approved": true,
                "data": result.data,
            })
            .to_string();
            let out = ExecutionStepOutput {
                step_id: step.id,
                agent_id: agent.id,
                content,
                status: ExecutionStatus::Completed,
            };
            return commit_step_output(
                loop_ctx,
                legacy,
                sprint5,
                run_key,
                step,
                step_index,
                Instant::now(),
                out,
            );
        }
        return interrupt_for_human(
            loop_ctx,
            legacy,
            step,
            step_index,
            hitl,
            recovery_enabled,
        );
    }

    sprint5.emit(TraceEventKind::StepStarted {
        run_id: run_key.to_string(),
        step_id: step.id.to_string(),
        step_index,
        agent_name: agent.name.clone(),
        agent_role: agent.role.clone(),
    });

    let step_started = Instant::now();
    let out = match execute_agent_for_step(
        agent_runtime,
        agent,
        step,
        loop_ctx,
        memory,
        legacy,
        sprint5,
        run_key,
        tool_runtime,
        tool_invoker.clone(),
        provider.clone(),
        provider_max_tokens,
        provider_temperature,
        retry_config.clone(),
        step_timeout,
        workflow_deadline,
    ) {
        Ok(output) => output,
        Err(err) => {
            if let Some(fallback_id) = step.fallback_step_id {
                if let Some(fallback_step) = all_steps.iter().find(|s| s.id == fallback_id) {
                    if let Some(fallback_agent) = agents.get(&fallback_step.agent_id) {
                        sprint5.emit(TraceEventKind::StepFallbackActivated {
                            run_id: run_key.to_string(),
                            step_id: step.id.to_string(),
                            primary_agent_name: agent.name.clone(),
                            fallback_agent_name: fallback_agent.name.clone(),
                        });
                        match execute_agent_for_step(
                            agent_runtime,
                            fallback_agent,
                            step,
                            loop_ctx,
                            memory,
                            legacy,
                            sprint5,
                            run_key,
                            tool_runtime,
                            tool_invoker,
                            provider,
                            provider_max_tokens,
                            provider_temperature,
                            retry_config,
                            step_timeout,
                            workflow_deadline,
                        ) {
                            Ok(fallback_out) => fallback_out,
                            Err(fallback_err) => {
                                return fail_step(
                                    loop_ctx,
                                    legacy,
                                    sprint5,
                                    run_key,
                                    step,
                                    step_index,
                                    workflow_started,
                                    step_started,
                                    recovery_enabled,
                                    fallback_err,
                                );
                            }
                        }
                    } else {
                        return fail_step(
                            loop_ctx,
                            legacy,
                            sprint5,
                            run_key,
                            step,
                            step_index,
                            workflow_started,
                            step_started,
                            recovery_enabled,
                            err,
                        );
                    }
                } else {
                    return fail_step(
                        loop_ctx,
                        legacy,
                        sprint5,
                        run_key,
                        step,
                        step_index,
                        workflow_started,
                        step_started,
                        recovery_enabled,
                        err,
                    );
                }
            } else {
                return fail_step(
                    loop_ctx,
                    legacy,
                    sprint5,
                    run_key,
                    step,
                    step_index,
                    workflow_started,
                    step_started,
                    recovery_enabled,
                    err,
                );
            }
        }
    };

    commit_step_output(
        loop_ctx,
        legacy,
        sprint5,
        run_key,
        step,
        step_index,
        step_started,
        out,
    )
}

fn commit_step_output(
    loop_ctx: &mut RunLoop<'_>,
    legacy: &TraceEmitter,
    sprint5: &mut TraceEventEmitter<'_>,
    run_key: &str,
    step: &StepDefinition,
    step_index: usize,
    step_started: Instant,
    out: ExecutionStepOutput,
) -> Result<(), WorkflowRunError> {
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
    loop_ctx.step_outputs.push(out);
    Ok(())
}

/// Runs validated steps in `order` sequence; halts on first agent failure with a partial record.
#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_sorted_steps(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    run_input: &str,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    provider: Option<Arc<dyn ModelProvider>>,
    provider_max_tokens: u32,
    provider_temperature: f32,
    exec_config: &ExecutionConfig,
    resume: Option<ResumeParams>,
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    let retry_config = exec_config
        .retry
        .clone()
        .or_else(|| {
            workflow
                .retry_policy
                .as_ref()
                .map(RetryConfig::from_rcs)
        });
    if let Some(ref r) = retry_config {
        r.validate().map_err(WorkflowRunError::Aborted)?;
    }
    exec_config.validate().map_err(WorkflowRunError::Aborted)?;
    let step_timeout = exec_config.timeouts.step_timeout;
    let workflow_timeout = exec_config.timeouts.workflow_timeout;
    let run_id = exec_config
        .run_id
        .or_else(|| resume.as_ref().map(|r| r.original_run_id))
        .unwrap_or_else(Uuid::new_v4);
    let run_key = run_id.to_string();
    let workflow_started = Instant::now();
    let workflow_deadline = workflow_timeout.map(|wt| workflow_started + wt);
    let effective_input = resume
        .as_ref()
        .map(|r| r.run_input.as_str())
        .unwrap_or(run_input);

    with_store(|store| {
        let mut steps = workflow.steps.clone();
        steps.sort_by_key(|s| s.order);
        let step_count = steps.len();
        let start_index = resume.as_ref().map(|r| r.start_step_index).unwrap_or(0);

        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), store);
        let mut legacy = TraceEmitter::new(run_id);

        if let Some(ref r) = resume {
            sprint5.emit(TraceEventKind::WorkflowRecoveryStarted {
                run_id: run_key.clone(),
                original_run_id: r.original_run_id.to_string(),
                resume_from_step: r.start_step_index,
            });
        } else {
            sprint5.emit(TraceEventKind::WorkflowStarted {
                run_id: run_key.clone(),
                workflow_name: workflow.name.clone(),
                step_count,
            });
            legacy.workflow_started();
        }
        info!(run_id = %run_id, workflow_id = %workflow.id, "workflow execution started");

        let mut state = StateEngine::new();
        let mut memory = MemoryCoordinator::new(run_id);
        let mut step_outputs = Vec::new();
        if let Some(ref r) = resume {
            for pre in &r.precompleted {
                state.commit(pre.clone()).map_err(|e: StateError| {
                    WorkflowRunError::Aborted(RuntimeError::StateCommitFailed {
                        step_id: pre.step_id,
                        reason: e.to_string(),
                    })
                })?;
                step_outputs.push(pre.clone());
            }
        }
        let mut loop_ctx = RunLoop {
            run_id,
            workflow_id: workflow.id,
            state: &mut state,
            step_outputs: &mut step_outputs,
            run_input: effective_input,
        };

        for (step_index, step) in steps.iter().enumerate() {
            if step_index < start_index {
                continue;
            }
            if let Err(err) = check_workflow_timeout(
                workflow_timeout,
                workflow_started,
                &run_key,
                &mut sprint5,
            ) {
                sprint5.emit(TraceEventKind::WorkflowFailed {
                    run_id: run_key.clone(),
                    duration_ms: workflow_started.elapsed().as_millis() as u64,
                    failed_step_index: Some(step_index),
                    error_code: "workflow_timeout".into(),
                });
                let partial = partial_record(&loop_ctx, &legacy);
                return Err(WorkflowRunError::Failed {
                    error: err,
                    partial,
                });
            }
            let Some(agent) = agents.get(&step.agent_id) else {
                return Err(WorkflowRunError::Aborted(RuntimeError::AgentNotFound {
                    agent_id: step.agent_id,
                    step_id: step.id,
                }));
            };
            let step_approval = resume
                .as_ref()
                .and_then(|r| r.approval.as_ref())
                .filter(|_| step_index == start_index);
            run_one_step(
                agent_runtime,
                &mut loop_ctx,
                step,
                step_index,
                agent,
                &steps,
                agents,
                &mut memory,
                &mut legacy,
                &mut sprint5,
                &run_key,
                tool_runtime,
                tool_invoker.clone(),
                workflow_started,
                provider.clone(),
                provider_max_tokens,
                provider_temperature,
                retry_config.clone(),
                step_timeout,
                workflow_deadline,
                exec_config.recovery_enabled,
                step_approval,
            )?;
        }

        if let Some(ref r) = resume {
            let re_executed = step_count.saturating_sub(r.start_step_index);
            sprint5.emit(TraceEventKind::WorkflowRecoveryCompleted {
                run_id: run_key.clone(),
                original_run_id: r.original_run_id.to_string(),
                steps_re_executed: re_executed,
            });
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
        maybe_export_trace(&run_key);
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
