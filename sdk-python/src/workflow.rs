//! Execute workflows via `WorkflowEngine`.

use std::sync::Arc;

use arcflow_core::constants::{
    ANTHROPIC_API_KEY_ENV, GEMINI_API_KEY_ENV, OPENAI_API_KEY_ENV,
};
use arcflow_core::get_execution_trace;
use arcflow_core::providers::ModelProvider;
use arcflow_core::providers::ProviderRuntime;
use arcflow_core::rcs::types::{ProviderConfig, ProviderId};
use arcflow_core::workflow::{WorkflowEngine, WorkflowExecutionRecord};
use pyo3::prelude::*;
use pyo3::types::PyList;
use uuid::Uuid;

use crate::errors::{configuration_error, workflow_run_error_to_py};
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

#[allow(clippy::type_complexity)]
fn provider_from_tuple(
    row: Option<(String, String, u32, f32)>,
) -> PyResult<(Option<Arc<dyn ModelProvider>>, u32, f32)> {
    let Some((kind, model, max_tokens, temperature)) = row else {
        return Ok((None, 0, 0.0));
    };
    let provider_id = match kind.as_str() {
        "openai" => ProviderId::OpenAI,
        "anthropic" => ProviderId::Anthropic,
        "gemini" => ProviderId::Gemini,
        other => {
            return Err(configuration_error(format!(
                "Unknown provider '{other}'. Use openai, anthropic, or gemini."
            )));
        }
    };
    let api_key_env = match provider_id {
        ProviderId::OpenAI => OPENAI_API_KEY_ENV,
        ProviderId::Anthropic => ANTHROPIC_API_KEY_ENV,
        ProviderId::Gemini => GEMINI_API_KEY_ENV,
        ProviderId::Custom => {
            return Err(configuration_error("Custom providers are not supported."));
        }
    };
    let config = ProviderConfig {
        provider_id,
        model,
        api_key_env: api_key_env.to_string(),
        params: None,
    };
    let provider = ProviderRuntime::from_config(&config)
        .map_err(|e| configuration_error(format!("[ArcFlow] {e}")))?;
    Ok((Some(provider), max_tokens, temperature))
}

#[pyfunction]
#[pyo3(signature = (workflow_name, workflow_id, agents, steps, run_input, tool_executors, provider = None))]
#[allow(clippy::result_large_err)] // WorkflowRunError carries partial record by design (ADR-002)
#[allow(clippy::too_many_arguments)]
pub fn execute_workflow(
    py: Python<'_>,
    workflow_name: String,
    workflow_id: String,
    agents: Bound<'_, PyList>,
    steps: Bound<'_, PyList>,
    run_input: String,
    tool_executors: Bound<'_, PyList>,
    provider: Option<(String, String, u32, f32)>,
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
    let (provider, max_tokens, temperature) = provider_from_tuple(provider)?;
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
            provider,
            max_tokens,
            temperature,
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
