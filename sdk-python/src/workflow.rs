//! Execute workflows via `WorkflowEngine`.

use std::sync::Arc;

use arcflow_core::get_execution_trace;
use arcflow_core::workflow::{WorkflowEngine, WorkflowExecutionRecord};
use pyo3::prelude::*;
use pyo3::types::PyList;
use uuid::Uuid;

use crate::errors::workflow_run_error_to_py;
use crate::tools::{register_tool_callables, PyToolInvoker};
use crate::types::{
    build_tool_runtime, build_workflow, parse_agent_tuple, parse_step_tuple, AgentInput,
};

#[pyclass(name = "WorkflowResult")]
#[derive(Clone)]
pub struct PyWorkflowResult {
    #[pyo3(get)]
    pub output: String,
    #[pyo3(get)]
    pub run_id: String,
    #[pyo3(get)]
    pub step_count: usize,
    #[pyo3(get)]
    pub trace_events_json: String,
}

fn record_to_py(record: WorkflowExecutionRecord) -> PyWorkflowResult {
    let output = record
        .step_outputs
        .last()
        .map(|s| s.content.clone())
        .unwrap_or_default();
    let trace_events_json =
        serde_json::to_string(&record.trace_events).unwrap_or_else(|_| "[]".to_string());
    PyWorkflowResult {
        output,
        run_id: record.run_id.to_string(),
        step_count: record.step_outputs.len(),
        trace_events_json,
    }
}

#[pyfunction]
#[pyo3(signature = (workflow_name, workflow_id, agents, steps, run_input, tool_executors))]
#[allow(clippy::result_large_err)] // WorkflowRunError carries partial record by design (ADR-002)
pub fn execute_workflow(
    py: Python<'_>,
    workflow_name: String,
    workflow_id: String,
    agents: Bound<'_, PyList>,
    steps: Bound<'_, PyList>,
    run_input: String,
    tool_executors: Bound<'_, PyList>,
) -> PyResult<PyWorkflowResult> {
    let wf_id = match Uuid::parse_str(&workflow_id) {
        Ok(id) => id,
        Err(_) => {
            return Err(crate::errors::configuration_error("Invalid workflow id."));
        }
    };
    let mut agent_inputs: Vec<AgentInput> = Vec::new();
    for item in agents.iter() {
        agent_inputs.push(parse_agent_tuple(item)?);
    }
    let mut step_inputs = Vec::new();
    for item in steps.iter() {
        step_inputs.push(parse_step_tuple(item)?);
    }
    let mut reg_pairs = Vec::new();
    let mut idx = 0;
    for agent in &agent_inputs {
        for tool in &agent.tools {
            let callable = tool_executors.get_item(idx)?.unbind();
            reg_pairs.push((tool.name.clone(), callable));
            idx += 1;
        }
    }
    register_tool_callables(py, reg_pairs)?;

    let (workflow, agent_map) =
        build_workflow(workflow_name, wf_id, agent_inputs.clone(), step_inputs)?;
    let tool_runtime = build_tool_runtime(&agent_inputs);
    let engine = WorkflowEngine::new();
    let invoker: Option<Arc<dyn arcflow_core::tools::ToolInvoker>> = if tool_runtime.has_tools() {
        Some(Arc::new(PyToolInvoker))
    } else {
        None
    };
    let record = py.allow_threads(|| {
        engine.execute_with_tools(
            &workflow,
            &agent_map,
            &run_input,
            if tool_runtime.has_tools() {
                Some(&tool_runtime)
            } else {
                None
            },
            invoker,
            None,
        )
    });
    let record = match record {
        Ok(record) => record,
        Err(err) => return Err(workflow_run_error_to_py(err)),
    };
    Ok(record_to_py(record))
}

#[pyfunction]
pub fn get_execution_trace_json(run_id: String) -> PyResult<String> {
    match get_execution_trace(&run_id) {
        Some(trace) => serde_json::to_string(&trace).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "[ArcFlow] Failed to serialize trace: {e}"
            ))
        }),
        None => Err(crate::errors::trace_not_found(&run_id)),
    }
}
