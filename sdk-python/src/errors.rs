//! Map `arcflow_core` errors to `arcflow.exceptions` types.

use arcflow_core::error::RuntimeError;
use arcflow_core::workflow::WorkflowRunError;
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
            let run_id = partial.run_id.to_string();
            let failed = partial.step_outputs.last().map(|o| o.agent_id.to_string());
            raise_execution(py, runtime_execution_message(&error), Some(run_id), failed)
        }
    })
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
        } => format!(
            "Tool '{tool_name}' failed for step '{step_id}': {reason}."
        ),
        RuntimeError::MemoryOperationFailed { reason } => {
            format!("Memory operation failed: {reason}.")
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
            tool_name,
            reason,
            ..
        } => format!("Tool '{tool_name}' failed: {reason}."),
        RuntimeError::MemoryOperationFailed { reason } => {
            format!("Memory operation failed: {reason}.")
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
