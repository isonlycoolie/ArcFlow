//! Live workflow stream iterator for Python SDK.

use std::sync::{Arc, Mutex};

use arcflow_core::streaming::{StreamEvent, StreamRunBridge};
use arcflow_core::workflow::{ExecutionConfig, StreamConfig, WorkflowEngine, WorkflowExecutionRecord};
use pyo3::prelude::*;
use pyo3::types::PyList;
use uuid::Uuid;

use crate::errors::{configuration_error, workflow_run_error_to_py};
use crate::execution_config::parse_execution_config;
use crate::tools::{register_tool_callables, PyToolInvoker};
use crate::types::{
    build_tool_runtime, build_workflow, parse_agent_tuple, parse_step_tuple, AgentInput,
};
use crate::workflow::{provider_from_tuple, record_to_py, PyWorkflowResult};

fn stream_event_to_dict(py: Python<'_>, event: StreamEvent) -> PyResult<Py<PyAny>> {
    let json = serde_json::to_string(&event).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "[ArcFlow] Failed to serialize stream event: {e}"
        ))
    })?;
    let parsed: serde_json::Value = serde_json::from_str(&json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "[ArcFlow] Failed to parse stream event: {e}"
        ))
    })?;
    let dict = parsed
        .as_object()
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "[ArcFlow] Stream event must be a JSON object.",
            )
        })?;
    let out = pyo3::types::PyDict::new_bound(py);
    for (key, value) in dict {
        out.set_item(key, json_value_to_py(py, value)?)?;
    }
    Ok(out.into())
}

fn json_value_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(v) => Ok(v.into_py(py)),
        serde_json::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(u) = v.as_u64() {
                Ok(u.into_py(py))
            } else if let Some(f) = v.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(v.to_string().into_py(py))
            }
        }
        serde_json::Value::String(v) => Ok(v.into_py(py)),
        serde_json::Value::Array(items) => {
            let list = PyList::empty_bound(py);
            for item in items {
                list.append(json_value_to_py(py, item)?)?;
            }
            Ok(list.into())
        }
        serde_json::Value::Object(map) => {
            let dict = pyo3::types::PyDict::new_bound(py);
            for (key, item) in map {
                dict.set_item(key, json_value_to_py(py, item)?)?;
            }
            Ok(dict.into())
        }
    }
}

#[pyclass(name = "WorkflowStreamIterator")]
pub struct PyWorkflowStreamIterator {
    bridge: Option<Mutex<StreamRunBridge<Result<WorkflowExecutionRecord, arcflow_core::workflow::WorkflowRunError>>>>,
    events_exhausted: bool,
}

#[pymethods]
impl PyWorkflowStreamIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if self.events_exhausted {
            return Err(PyErr::new::<pyo3::exceptions::PyStopIteration, _>(
                "stream finished",
            ));
        }
        let bridge = self.bridge.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyStopIteration, _>("stream finished")
        })?;
        let event = py.allow_threads(|| bridge.lock().expect("stream lock").recv_event());
        match event {
            Some(ev) => stream_event_to_dict(py, ev),
            None => {
                self.events_exhausted = true;
                Err(PyErr::new::<pyo3::exceptions::PyStopIteration, _>(
                    "stream finished",
                ))
            }
        }
    }

    fn finalize(&mut self, py: Python<'_>) -> PyResult<PyWorkflowResult> {
        let bridge = self
            .bridge
            .take()
            .ok_or_else(|| configuration_error("Stream iterator already finalized."))?;
        self.events_exhausted = true;
        let outcome = py.allow_threads(|| bridge.into_inner().expect("stream lock").join());
        match outcome.result {
            Ok(record) => Ok(record_to_py(record)),
            Err(err) => Err(workflow_run_error_to_py(err)),
        }
    }
}

#[pyfunction]
#[pyo3(signature = (workflow_name, workflow_id, agents, steps, run_input, tool_executors, provider = None, exec_config_json = None, graph_json = None))]
#[allow(clippy::too_many_arguments)]
pub fn start_workflow_stream(
    py: Python<'_>,
    workflow_name: String,
    workflow_id: String,
    agents: Bound<'_, PyList>,
    steps: Bound<'_, PyList>,
    run_input: String,
    tool_executors: Bound<'_, PyList>,
    provider: Option<(String, String, u32, f32)>,
    exec_config_json: Option<String>,
    graph_json: Option<String>,
) -> PyResult<PyWorkflowStreamIterator> {
    let wf_id = Uuid::parse_str(&workflow_id)
        .map_err(|_| configuration_error("Invalid workflow id."))?;
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

    let (mut workflow, agent_map) =
        build_workflow(workflow_name, wf_id, agent_inputs.clone(), step_inputs)?;
    if let Some(raw) = graph_json {
        crate::graph::apply_graph_json(&mut workflow, &raw).map_err(configuration_error)?;
    }
    let tool_runtime = build_tool_runtime(&agent_inputs);
    let invoker: Option<Arc<dyn arcflow_core::tools::ToolInvoker>> = if tool_runtime.has_tools() {
        Some(Arc::new(PyToolInvoker))
    } else {
        None
    };
    let (provider, max_tokens, temperature) = provider_from_tuple(provider)?;
    let mut exec_config: ExecutionConfig = parse_execution_config(exec_config_json.as_deref())
        .map_err(configuration_error)?;
    exec_config.stream = Some(StreamConfig { enabled: true });
    let engine = WorkflowEngine::new();

    let bridge = py.allow_threads(|| {
        StreamRunBridge::spawn(move |stream_tx| {
            engine.execute_with_config(
                &workflow,
                &agent_map,
                &run_input,
                if tool_runtime.has_tools() {
                    Some(&tool_runtime)
                } else {
                    None
                },
                invoker,
                provider,
                max_tokens,
                temperature,
                &exec_config,
                Some(stream_tx),
            )
        })
    });

    Ok(PyWorkflowStreamIterator {
        bridge: Some(Mutex::new(bridge)),
        events_exhausted: false,
    })
}
