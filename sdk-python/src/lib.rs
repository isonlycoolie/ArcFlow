//! PyO3 bindings for ArcFlow — translate only; orchestration lives in `arcflow_core`.

#![allow(clippy::useless_conversion)] // PyO3 `PyResult<T>` triggers a false positive on some toolchains

mod errors;
mod types;
mod workflow;

use pyo3::prelude::*;

use workflow::{execute_workflow, PyWorkflowResult};

/// Native extension module (import as `arcflow._arcflow_binding`).
#[pymodule]
fn _arcflow_binding(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(execute_workflow, m)?)?;
    m.add_class::<PyWorkflowResult>()?;
    Ok(())
}
