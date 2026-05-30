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
