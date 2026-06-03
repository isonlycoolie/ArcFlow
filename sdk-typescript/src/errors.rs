//! Map `arcflow_core` errors to napi errors consumed by the TS SDK.

use arcflow_core::error::RuntimeError;
use arcflow_core::workflow::{WorkflowExecutionRecord, WorkflowRunError};
use napi::Error;
use uuid::Uuid;

fn prefix(msg: &str) -> String {
    if msg.starts_with("[ArcFlow]") {
        msg.to_string()
    } else {
        format!("[ArcFlow] {msg}")
    }
}

pub fn configuration_error(message: impl Into<String>) -> Error {
    Error::from_reason(prefix(&message.into()))
}

pub fn trace_not_found(run_id: &str) -> Error {
    Error::from_reason(prefix(&format!(
        "No trace found for run '{run_id}'. \
         Run workflow.run() first or the trace may have been evicted."
    )))
}

pub fn parse_uuid(field: &str, value: &str) -> Result<Uuid, Error> {
    Uuid::parse_str(value)
        .map_err(|_| configuration_error(format!("Invalid UUID for {field}: '{value}'.")))
}

pub fn workflow_run_error_to_napi(err: WorkflowRunError) -> Error {
    match err {
        WorkflowRunError::Aborted(inner) => configuration_error(runtime_config_message(&inner)),
        WorkflowRunError::Failed { error, partial } => {
            raise_for_runtime_error(&error, partial.run_id.to_string(), &partial)
        }
        WorkflowRunError::Interrupted { partial, .. } => Error::from_reason(format!(
            "WorkflowInterruptedError|{}|{}",
            partial.run_id, "Workflow paused for human approval."
        )),
    }
}

fn raise_for_runtime_error(
    err: &RuntimeError,
    run_id: String,
    partial: &WorkflowExecutionRecord,
) -> Error {
    match err {
        RuntimeError::ProviderCallFailed {
            provider_id,
            step_id,
            reason,
        } => Error::from_reason(prefix(&format!(
            "ProviderExecutionError|{provider_id}|{run_id}|{step_id}|Provider '{provider_id}' \
             failed for step '{step_id}': {reason}."
        ))),
        RuntimeError::ToolExecutionFailed { .. }
        | RuntimeError::MemoryOperationFailed { .. }
        | RuntimeError::InfrastructureUnavailable { .. } => {
            Error::from_reason(runtime_execution_message(err))
        }
        _ => {
            let _failed = partial.step_outputs.last().map(|o| o.agent_id.to_string());
            Error::from_reason(format!(
                "WorkflowExecutionError|{run_id}|{}|{}",
                partial
                    .step_outputs
                    .last()
                    .map(|o| o.agent_id.to_string())
                    .unwrap_or_default(),
                runtime_execution_message(err)
            ))
        }
    }
}

fn runtime_config_message(err: &RuntimeError) -> String {
    prefix(&match err {
        RuntimeError::InvalidWorkflowDefinition { reason } => format!(
            "Workflow definition is invalid: {reason}. \
             Check workflow name and step definitions."
        ),
        RuntimeError::AgentNotFound { .. } => {
            "An agent referenced by a step was not registered.".into()
        }
        RuntimeError::StateCommitFailed { step_id, reason } => {
            format!("State commit failed for step '{step_id}': {reason}.")
        }
        RuntimeError::AgentExecutionFailed { step_id, reason } => {
            format!("Agent execution failed for step '{step_id}': {reason}.")
        }
        RuntimeError::ToolExecutionFailed {
            tool_name,
            step_id,
            reason,
        } => format!("Tool '{tool_name}' failed for step '{step_id}': {reason}."),
        RuntimeError::MemoryOperationFailed { step_id, reason } => {
            format!("Memory operation failed for step '{step_id}': {reason}.")
        }
        RuntimeError::InfrastructureUnavailable {
            backend,
            suggestion,
            step_id,
        } => format!("Infrastructure unavailable ({backend}) for step '{step_id}': {suggestion}."),
        RuntimeError::ProviderCallFailed {
            provider_id,
            step_id,
            reason,
        } => format!("Provider '{provider_id}' failed for step '{step_id}': {reason}."),
        RuntimeError::StepTimeout { .. }
        | RuntimeError::WorkflowTimeout { .. }
        | RuntimeError::RetryExhausted { .. }
        | RuntimeError::RecoveryStorageError { .. } => format!("{err}."),
        RuntimeError::HumanRejected { approval_key } => {
            format!("Human approval rejected for key '{approval_key}'.")
        }
        RuntimeError::HumanTimeout { approval_key } => {
            format!("Human approval timed out for key '{approval_key}'.")
        }
        RuntimeError::ApprovalNotFound { approval_key } => {
            format!("Approval key '{approval_key}' was not found.")
        }
        RuntimeError::AlreadyApproved { approval_key } => {
            format!("Approval key '{approval_key}' was already resolved.")
        }
    })
}

fn runtime_execution_message(err: &RuntimeError) -> String {
    prefix(&match err {
        RuntimeError::AgentExecutionFailed { reason, .. } => {
            format!("Step execution failed: {reason}.")
        }
        RuntimeError::StateCommitFailed { step_id, reason } => {
            format!("State commit failed for step '{step_id}': {reason}.")
        }
        RuntimeError::InvalidWorkflowDefinition { reason } => {
            format!("Workflow failed during execution: {reason}.")
        }
        RuntimeError::AgentNotFound { .. } => {
            "Execution stopped because a step references an unknown agent.".into()
        }
        RuntimeError::ToolExecutionFailed {
            tool_name, reason, ..
        } => format!("Tool '{tool_name}' failed: {reason}."),
        RuntimeError::MemoryOperationFailed { step_id, reason } => {
            format!("Memory operation failed for step '{step_id}': {reason}.")
        }
        RuntimeError::InfrastructureUnavailable {
            backend,
            suggestion,
            step_id,
        } => format!("Infrastructure unavailable ({backend}) for step '{step_id}': {suggestion}."),
        RuntimeError::ProviderCallFailed {
            provider_id,
            reason,
            ..
        } => format!("Provider '{provider_id}' failed: {reason}."),
        RuntimeError::StepTimeout {
            step_id,
            configured_ms,
            ..
        } => {
            format!("Step '{step_id}' timed out (limit {configured_ms}ms).")
        }
        RuntimeError::WorkflowTimeout { configured_ms, .. } => {
            format!("Workflow timed out (limit {configured_ms}ms).")
        }
        RuntimeError::RetryExhausted {
            step_id,
            attempts_made,
            last_error_code,
        } => format!("Step '{step_id}' failed after {attempts_made} attempts: {last_error_code}."),
        RuntimeError::RecoveryStorageError { reason } => {
            format!("Recovery storage error: {reason}.")
        }
        RuntimeError::HumanRejected { approval_key } => {
            format!("Human approval rejected for key '{approval_key}'.")
        }
        RuntimeError::HumanTimeout { approval_key } => {
            format!("Human approval timed out for key '{approval_key}'.")
        }
        RuntimeError::ApprovalNotFound { approval_key } => {
            format!("Approval key '{approval_key}' was not found.")
        }
        RuntimeError::AlreadyApproved { approval_key } => {
            format!("Approval key '{approval_key}' was already resolved.")
        }
    })
}
