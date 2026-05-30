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
use crate::streaming::{StreamChannelSender, StreamEvent};
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

fn try_emit_stream(tx: &Option<StreamChannelSender>, event: StreamEvent) {
    if let Some(sender) = tx {
        sender.try_send(event);
    }
}

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
    pub(crate) test_config: Option<crate::workflow::TestConfig>,
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
    stream_tx: &Option<StreamChannelSender>,
) -> Result<(), WorkflowRunError> {
    try_emit_stream(
        stream_tx,
        StreamEvent::Error {
            code: "step_failed".into(),
            message: err.to_string(),
            step_id: step.id.to_string(),
        },
    );
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
    stream_tx: &Option<StreamChannelSender>,
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
        step_order: step.order,
        test_config: loop_ctx.test_config.clone(),
        test_attempt: 1,
        stream_tx: stream_tx.clone(),
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
    stream_tx: Option<StreamChannelSender>,
    node_id: Option<String>,
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
                    &stream_tx,
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
                &stream_tx,
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
