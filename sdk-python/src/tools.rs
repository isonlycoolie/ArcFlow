//! Python tool callable registry and invoker.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use arcflow_core::tools::ToolError;
use arcflow_core::tools::ToolInvoker;
use pyo3::prelude::*;
use serde_json::Value;

static TOOL_CALLABLES: OnceLock<Mutex<HashMap<String, Py<PyAny>>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, Py<PyAny>>> {
    TOOL_CALLABLES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Registers callables for one workflow run (clears previous entries).
pub fn register_tool_callables(_py: Python<'_>, tools: Vec<(String, Py<PyAny>)>) -> PyResult<()> {
    let mut guard = registry()
        .lock()
        .map_err(|_| crate::errors::configuration_error("Tool registry lock poisoned."))?;
    guard.clear();
    for (name, callable) in tools {
        guard.insert(name, callable);
    }
    Ok(())
}

pub struct PyToolInvoker;

impl ToolInvoker for PyToolInvoker {
    fn invoke(&self, name: &str, input: &Value) -> Result<String, ToolError> {
        Python::with_gil(|py| invoke_tool_py(py, name, input))
    }
}

fn invoke_tool_py(py: Python<'_>, name: &str, input: &Value) -> Result<String, ToolError> {
    let guard = registry().lock().map_err(|_| ToolError::ExecutionFailed {
        name: name.to_string(),
        step_id: None,
        reason: "tool registry lock poisoned".into(),
    })?;
    let Some(callable) = guard.get(name) else {
        return Err(ToolError::NotRegistered {
            name: name.to_string(),
        });
    };
    let json_mod = py.import("json").map_err(|e| map_py_err(name, e))?;
    let payload = serde_json::to_string(input).map_err(|e| ToolError::ExecutionFailed {
        name: name.to_string(),
        step_id: None,
        reason: e.to_string(),
    })?;
    let py_input = json_mod
        .call_method1("loads", (payload,))
        .map_err(|e| map_py_err(name, e))?;
    let result = callable
        .call1(py, (py_input,))
        .map_err(|e| map_py_err(name, e))?;
    result
        .extract::<String>(py)
        .map_err(|e| ToolError::ExecutionFailed {
            name: name.to_string(),
            step_id: None,
            reason: e.to_string(),
        })
}

fn map_py_err(name: &str, err: PyErr) -> ToolError {
    ToolError::ExecutionFailed {
        name: name.to_string(),
        step_id: None,
        reason: err.to_string(),
    }
}
