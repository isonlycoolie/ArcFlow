//! Map `arcflow_core` errors to `arcflow.exceptions` types.

use arcflow_core::error::RuntimeError;
use arcflow_core::workflow::{WorkflowExecutionRecord, WorkflowRunError};
use pyo3::prelude::*;
use uuid::Uuid;

fn prefix(msg: &str) -> String {
    if msg.starts_with("[ArcFlow]") {
        msg.to_string()
    } else {
        format!("[ArcFlow] {msg}")
    }
}

fn import_exceptions(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
    py.import("arcflow.exceptions")
}

pub fn workflow_run_error_to_py(err: WorkflowRunError) -> PyErr {
    Python::with_gil(|py| match err {
        WorkflowRunError::Aborted(inner) => raise_configuration(py, runtime_config_message(&inner)),
        WorkflowRunError::Failed { error, partial } => {
            raise_for_runtime_error(py, &error, partial.run_id.to_string(), &partial)
        }
        WorkflowRunError::Interrupted {
            approval_key,
            expires_at,
            partial,
        } => raise_interrupted(
            py,
            partial.run_id.to_string(),
            approval_key,
            expires_at.to_rfc3339(),
        ),
    })
}

fn raise_interrupted(
    py: Python<'_>,
    run_id: String,
    approval_key: String,
    expires_at: String,
) -> PyErr {
    use pyo3::types::PyDict;
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let hitl_mod = py.import("arcflow.hitl")?;
        let cls = hitl_mod.getattr("WorkflowInterruptedError")?;
        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("run_id", run_id.clone())?;
        kwargs.set_item("approval_key", approval_key.clone())?;
        kwargs.set_item("expires_at", expires_at)?;
        cls.call(
            (prefix(&format!(
                "Workflow paused for human approval '{approval_key}'."
            )),),
            Some(&kwargs),
        )
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_for_runtime_error(
    py: Python<'_>,
    err: &RuntimeError,
    run_id: String,
    partial: &WorkflowExecutionRecord,
) -> PyErr {
    match err {
        RuntimeError::ToolExecutionFailed {
            tool_name,
            step_id,
            reason,
        } => raise_tool_execution(
            py,
            prefix(&format!(
                "Tool '{tool_name}' failed for step '{step_id}': {reason}. \
                 Check your tool execute function and input schema."
            )),
            Some(tool_name.clone()),
            Some(run_id),
            Some(step_id.to_string()),
        ),
        RuntimeError::InfrastructureUnavailable {
            backend,
            suggestion,
            step_id,
        } => raise_infrastructure(
            py,
            prefix(&format!(
                "Infrastructure unavailable ({backend}) for step '{step_id}'. {suggestion}"
            )),
            backend.clone(),
            suggestion.clone(),
        ),
        RuntimeError::MemoryOperationFailed { step_id, reason } => raise_memory_operation(
            py,
            prefix(&format!(
                "Memory operation failed for step '{step_id}': {reason}. \
                 Check memory configuration and backend connectivity."
            )),
        ),
        RuntimeError::ProviderCallFailed {
            provider_id,
            step_id,
            reason,
        } => raise_provider_execution(
            py,
            prefix(&format!(
                "Provider '{provider_id}' failed for step '{step_id}': {reason}."
            )),
            Some(provider_id.clone()),
            Some(run_id),
            Some(step_id.to_string()),
        ),
        RuntimeError::RetryExhausted {
            step_id,
            attempts_made,
            last_error_code,
        } => raise_retry_exhausted(
            py,
            prefix(&format!(
                "Retry exhausted for step '{step_id}' after {attempts_made} attempts. \
                 Last error: {last_error_code}. Check provider availability."
            )),
            Some(run_id),
            Some(step_id.clone()),
            *attempts_made,
            Some(last_error_code.clone()),
        ),
        RuntimeError::StepTimeout {
            step_id,
            configured_ms,
            elapsed_ms,
        } => raise_workflow_timeout(
            py,
            prefix(&format!(
                "Step '{step_id}' timed out after {elapsed_ms}ms (limit {configured_ms}ms). \
                 Increase step_timeout or optimize the step."
            )),
            Some(run_id),
            Some(step_id.clone()),
            "step",
            *configured_ms as f64 / 1000.0,
            *elapsed_ms as f64 / 1000.0,
        ),
        RuntimeError::WorkflowTimeout {
            configured_ms,
            elapsed_ms,
        } => raise_workflow_timeout(
            py,
            prefix(&format!(
                "Workflow timed out after {elapsed_ms}ms (limit {configured_ms}ms). \
                 Increase timeout or optimize steps."
            )),
            Some(run_id),
            None,
            "workflow",
            *configured_ms as f64 / 1000.0,
            *elapsed_ms as f64 / 1000.0,
        ),
        RuntimeError::HumanRejected { approval_key } => {
            let built: PyResult<Bound<'_, PyAny>> = (|| {
                let hitl_mod = py.import("arcflow.hitl")?;
                let cls = hitl_mod.getattr("HumanRejectedError")?;
                let kwargs = pyo3::types::PyDict::new_bound(py);
                kwargs.set_item("approval_key", approval_key)?;
                cls.call(
                    (prefix(&format!(
                        "Human rejected approval '{approval_key}'."
                    )),),
                    Some(&kwargs),
                )
            })();
            match built {
                Ok(value) => PyErr::from_value(value),
                Err(err) => err,
            }
        }
        _ => {
            let failed = partial.step_outputs.last().map(|o| o.agent_id.to_string());
            raise_execution(py, runtime_execution_message(err), Some(run_id), failed)
        }
    }
}

fn runtime_config_message(err: &RuntimeError) -> String {
    prefix(&match err {
        RuntimeError::InvalidWorkflowDefinition { reason } => format!(
            "Workflow definition is invalid: {reason}. \
             Check workflow name and step definitions."
        ),
        RuntimeError::AgentNotFound { .. } => "An agent referenced by a step was not registered. \
             Ensure each step uses an Agent passed to workflow.step()."
            .into(),
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
            format!("Human rejected approval '{approval_key}'.")
        }
        RuntimeError::HumanTimeout { approval_key } => {
            format!("Human approval '{approval_key}' timed out.")
        }
        RuntimeError::ApprovalNotFound { approval_key } => {
            format!("Approval '{approval_key}' not found.")
        }
        RuntimeError::AlreadyApproved { approval_key } => {
            format!("Approval '{approval_key}' was already resolved.")
        }
    })
}

fn runtime_execution_message(err: &RuntimeError) -> String {
    prefix(&match err {
        RuntimeError::AgentExecutionFailed { reason, .. } => format!(
            "Step execution failed: {reason}. \
             Review agent role and instructions or retry the run."
        ),
        RuntimeError::StateCommitFailed { step_id, reason } => format!(
            "State commit failed for step '{step_id}': {reason}. \
             This indicates an internal runtime issue."
        ),
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
        } => format!("Step '{step_id}' timed out (limit {configured_ms}ms)."),
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
            format!("Human rejected approval '{approval_key}'.")
        }
        RuntimeError::HumanTimeout { approval_key } => {
            format!("Human approval '{approval_key}' timed out.")
        }
        RuntimeError::ApprovalNotFound { approval_key } => {
            format!("Approval '{approval_key}' not found.")
        }
        RuntimeError::AlreadyApproved { approval_key } => {
            format!("Approval '{approval_key}' was already resolved.")
        }
    })
}

fn raise_configuration(py: Python<'_>, message: String) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("WorkflowConfigurationError")?;
        cls.call1((message,))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_execution(
    py: Python<'_>,
    message: String,
    run_id: Option<String>,
    failed_step: Option<String>,
) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("WorkflowExecutionError")?;
        cls.call1((message, run_id, failed_step))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_tool_execution(
    py: Python<'_>,
    message: String,
    tool_name: Option<String>,
    run_id: Option<String>,
    failed_step: Option<String>,
) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("ToolExecutionError")?;
        cls.call1((message, tool_name, run_id, failed_step))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_memory_operation(py: Python<'_>, message: String) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("MemoryOperationError")?;
        cls.call1((message,))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_infrastructure(
    py: Python<'_>,
    message: String,
    backend: String,
    suggestion: String,
) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("InfrastructureUnavailableError")?;
        cls.call1((message, backend, suggestion))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_provider_execution(
    py: Python<'_>,
    message: String,
    provider_id: Option<String>,
    run_id: Option<String>,
    failed_step: Option<String>,
) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("ProviderExecutionError")?;
        cls.call1((message, provider_id, run_id, failed_step))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_retry_exhausted(
    py: Python<'_>,
    message: String,
    run_id: Option<String>,
    failed_step: Option<String>,
    attempts_made: u32,
    last_error_code: Option<String>,
) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("RetryExhaustedError")?;
        cls.call1((message, attempts_made, run_id, failed_step, last_error_code))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

fn raise_workflow_timeout(
    py: Python<'_>,
    message: String,
    run_id: Option<String>,
    failed_step: Option<String>,
    timeout_type: &str,
    configured_seconds: f64,
    elapsed_seconds: f64,
) -> PyErr {
    let built: PyResult<Bound<'_, PyAny>> = (|| {
        let exc_mod = import_exceptions(py)?;
        let cls = exc_mod.getattr("WorkflowTimeoutError")?;
        cls.call1((
            message,
            timeout_type,
            configured_seconds,
            elapsed_seconds,
            run_id,
            failed_step,
        ))
    })();
    match built {
        Ok(value) => PyErr::from_value(value),
        Err(err) => err,
    }
}

pub fn parse_uuid(field: &str, value: &str) -> Result<Uuid, PyErr> {
    match Uuid::parse_str(value) {
        Ok(id) => Ok(id),
        Err(_) => Err(configuration_error(format!(
            "Invalid UUID for {field}: '{value}'."
        ))),
    }
}

pub fn configuration_error(message: impl Into<String>) -> PyErr {
    Python::with_gil(|py| raise_configuration(py, prefix(&message.into())))
}

pub fn trace_not_found(run_id: &str) -> PyErr {
    Python::with_gil(|py| {
        let built: PyResult<Bound<'_, PyAny>> = (|| {
            let exc_mod = import_exceptions(py)?;
            let cls = exc_mod.getattr("TraceNotFoundError")?;
            cls.call1((prefix(&format!(
                "No trace found for run '{run_id}'. \
                 Run workflow.run() first or the trace may have been evicted."
            )),))
        })();
        match built {
            Ok(value) => PyErr::from_value(value),
            Err(err) => err,
        }
    })
}
