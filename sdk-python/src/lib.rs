//! PyO3 bindings for ArcFlow — translate only; orchestration lives in `arcflow_core`.

#![allow(clippy::useless_conversion)] // PyO3 `PyResult<T>` triggers a false positive on some toolchains

mod errors;
mod execution_config;
mod graph;
mod memory;
mod stream;
mod tools;
mod types;
mod workflow;

use pyo3::prelude::*;

use memory::PyVectorStore;
use stream::{start_workflow_stream, PyWorkflowStreamIterator};
use workflow::{
    execute_resume_with_approval, execute_resume_workflow, execute_workflow,
    get_execution_trace_json, PyWorkflowResult,
};

/// Native extension module (import as `arcflow._arcflow_binding`).
#[pymodule]
fn _arcflow_binding(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(execute_workflow, m)?)?;
    m.add_function(wrap_pyfunction!(start_workflow_stream, m)?)?;
    m.add_function(wrap_pyfunction!(execute_resume_workflow, m)?)?;
    m.add_function(wrap_pyfunction!(execute_resume_with_approval, m)?)?;
    m.add_function(wrap_pyfunction!(get_execution_trace_json, m)?)?;
    m.add_class::<PyWorkflowResult>()?;
    m.add_class::<PyWorkflowStreamIterator>()?;
    m.add_class::<PyVectorStore>()?;
    Ok(())
}
