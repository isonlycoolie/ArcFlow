//! Execute workflows via `WorkflowEngine`.

use arcflow_core::workflow::{WorkflowEngine, WorkflowExecutionRecord};
use pyo3::prelude::*;
use pyo3::types::PyList;
use uuid::Uuid;

use crate::errors::workflow_run_error_to_py;
use crate::types::{build_workflow, parse_agent_tuple, parse_step_tuple};

#[pyclass(name = "WorkflowResult")]
#[derive(Clone)]
pub struct PyWorkflowResult {
    #[pyo3(get)]
    pub output: String,
    #[pyo3(get)]
    pub run_id: String,
    #[pyo3(get)]
    pub step_count: usize,
}

fn record_to_py(record: WorkflowExecutionRecord) -> PyWorkflowResult {
    let output = record
        .step_outputs
        .last()
        .map(|s| s.content.clone())
        .unwrap_or_default();
    PyWorkflowResult {
        output,
        run_id: record.run_id.to_string(),
        step_count: record.step_outputs.len(),
    }
}

#[pyfunction]
#[pyo3(signature = (workflow_name, workflow_id, agents, steps, run_input))]
pub fn execute_workflow(
    workflow_name: String,
    workflow_id: String,
    agents: Bound<'_, PyList>,
    steps: Bound<'_, PyList>,
    run_input: String,
) -> PyResult<PyWorkflowResult> {
    let wf_id = match Uuid::parse_str(&workflow_id) {
        Ok(id) => id,
        Err(_) => {
            return Err(crate::errors::configuration_error("Invalid workflow id."));
        }
    };
    let mut agent_inputs = Vec::new();
    for item in agents.iter() {
        agent_inputs.push(parse_agent_tuple(item)?);
    }
    let mut step_inputs = Vec::new();
    for item in steps.iter() {
        step_inputs.push(parse_step_tuple(item)?);
    }
    let (workflow, agent_map) = build_workflow(workflow_name, wf_id, agent_inputs, step_inputs)?;
    let engine = WorkflowEngine::new();
    let record = match engine.execute(&workflow, &agent_map, &run_input) {
        Ok(record) => record,
        Err(err) => return Err(workflow_run_error_to_py(err)),
    };
    Ok(record_to_py(record))
}
