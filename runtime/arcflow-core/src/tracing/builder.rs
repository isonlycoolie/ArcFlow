//! Assembles raw events into ExecutionTrace (Sprint 5).

use chrono::Utc;

use super::events::TraceEventKind;
use super::types::{
    ExecutionStatus, ExecutionTrace, MemoryOperation, MemoryOperationTrace, StepError,
    StepExecutionStatus, StepTrace, TokenUsage, ToolCallStatus, ToolCallTrace, TraceEvent,
};

/// Assembles a structured ExecutionTrace from stored events.
#[derive(Debug, Default)]
pub struct ExecutionTraceBuilder;

impl ExecutionTraceBuilder {
    /// Builds a trace; always returns a (possibly partial) snapshot.
    pub fn build(run_id: &str, events: &[TraceEvent], events_dropped: u32) -> ExecutionTrace {
        let mut workflow_name = String::from("unknown");
        let mut status = ExecutionStatus::Partial;
        let mut started_at = Utc::now();
        let mut completed_at = None;
        let mut total_tokens = TokenUsage::default();
        let mut steps: Vec<StepTrace> = Vec::new();
        let mut current: Option<StepTrace> = None;

        for event in events {
            apply_event(
                event,
                &mut workflow_name,
                &mut status,
                &mut started_at,
                &mut completed_at,
                &mut total_tokens,
                &mut steps,
                &mut current,
            );
        }
        if let Some(step) = current {
            steps.push(step);
        }
        if total_tokens.total_tokens == 0 && !steps.is_empty() {
            total_tokens = steps
                .iter()
                .fold(TokenUsage::default(), |acc, s| acc.add(&s.tokens));
        }
        let duration_ms =
            completed_at.map(|end| (end - started_at).num_milliseconds().max(0) as u64);

        ExecutionTrace {
            run_id: run_id.to_string(),
            workflow_name,
            status,
            started_at,
            completed_at,
            duration_ms,
            total_tokens,
            steps,
            events: events.to_vec(),
            is_complete: events_dropped == 0,
            events_dropped,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_event(
    event: &TraceEvent,
    workflow_name: &mut String,
    status: &mut ExecutionStatus,
    started_at: &mut chrono::DateTime<Utc>,
    completed_at: &mut Option<chrono::DateTime<Utc>>,
    total_tokens: &mut TokenUsage,
    steps: &mut Vec<StepTrace>,
    current: &mut Option<StepTrace>,
) {
    match &event.kind {
        TraceEventKind::WorkflowStarted {
            workflow_name: name,
            ..
        } => {
            *workflow_name = name.clone();
            *started_at = event.timestamp;
        }
        TraceEventKind::WorkflowCompleted {
            total_tokens: tokens,
            ..
        } => {
            *status = ExecutionStatus::Completed;
            *completed_at = Some(event.timestamp);
            *total_tokens = tokens.clone();
        }
        TraceEventKind::WorkflowFailed { .. } => {
            *status = ExecutionStatus::Failed;
            *completed_at = Some(event.timestamp);
        }
        TraceEventKind::StepStarted {
            step_id,
            step_index,
            agent_name,
            agent_role,
            ..
        } => {
            if let Some(prev) = current.take() {
                steps.push(prev);
            }
            *current = Some(StepTrace {
                step_index: *step_index,
                step_id: step_id.clone(),
                agent_name: agent_name.clone(),
                agent_role: agent_role.clone(),
                status: StepExecutionStatus::InProgress,
                started_at: event.timestamp,
                completed_at: None,
                duration_ms: None,
                tokens: TokenUsage::default(),
                tool_calls: Vec::new(),
                memory_operations: Vec::new(),
                error: None,
            });
        }
        TraceEventKind::StepCompleted {
            duration_ms,
            tokens,
            ..
        } => {
            if let Some(ref mut step) = current {
                step.completed_at = Some(event.timestamp);
                step.duration_ms = Some(*duration_ms);
                step.tokens = tokens.clone();
                step.status = StepExecutionStatus::Completed;
            }
        }
        TraceEventKind::StepFailed {
            error_code,
            error_message,
            duration_ms,
            ..
        } => {
            if let Some(ref mut step) = current {
                step.completed_at = Some(event.timestamp);
                step.duration_ms = Some(*duration_ms);
                step.status = StepExecutionStatus::Failed;
                step.error = Some(StepError {
                    error_code: error_code.clone(),
                    message: error_message.clone(),
                });
            }
        }
        TraceEventKind::ToolCallCompleted {
            tool_name,
            duration_ms,
            output_size_bytes,
            ..
        } => {
            if let Some(ref mut step) = current {
                step.tool_calls.push(ToolCallTrace {
                    tool_name: tool_name.clone(),
                    status: ToolCallStatus::Success,
                    duration_ms: *duration_ms,
                    input_schema_hash: String::new(),
                    output_size_bytes: Some(*output_size_bytes),
                    error_code: None,
                });
            }
        }
        TraceEventKind::MemoryRead {
            memory_type,
            key,
            hit,
            duration_ms,
            ..
        } => {
            if let Some(ref mut step) = current {
                step.memory_operations.push(MemoryOperationTrace {
                    operation: MemoryOperation::Read,
                    memory_type: memory_type.clone(),
                    key: key.clone(),
                    hit: Some(*hit),
                    duration_ms: *duration_ms,
                });
            }
        }
        TraceEventKind::MemoryWrite {
            memory_type,
            key,
            duration_ms,
            ..
        } => {
            if let Some(ref mut step) = current {
                step.memory_operations.push(MemoryOperationTrace {
                    operation: MemoryOperation::Write,
                    memory_type: memory_type.clone(),
                    key: key.clone(),
                    hit: None,
                    duration_ms: *duration_ms,
                });
            }
        }
        _ => {}
    }
}
