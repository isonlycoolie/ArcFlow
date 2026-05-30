//! Execute workflows via `WorkflowEngine`.

use std::sync::Arc;

use arcflow_core::constants::{
    ANTHROPIC_API_KEY_ENV, GEMINI_API_KEY_ENV, OPENAI_API_KEY_ENV,
};
use arcflow_core::get_execution_trace;
use arcflow_core::providers::{ModelProvider, ProviderRuntime};
use arcflow_core::rcs::types::{ProviderConfig, ProviderId};
use arcflow_core::workflow::{ExecutionConfig, StreamConfig, WorkflowEngine, WorkflowExecutionRecord};
use arcflow_core::streaming::{default_stream_pair, StreamEvent};
use napi::bindgen_prelude::*;
use napi::Error;
use napi_derive::napi;
use uuid::Uuid;

use crate::errors::{configuration_error, trace_not_found, workflow_run_error_to_napi};
use crate::execution_config::parse_execution_config;
use crate::types::{build_workflow, JsAgentInput, JsStepInput};

#[napi(object)]
pub struct JsProviderInput {
    pub kind: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

#[napi(object)]
pub struct JsWorkflowResult {
    pub output: String,
    pub run_id: String,
    pub step_count: u32,
    pub trace_events_json: String,
}

pub(crate) fn record_to_js(record: WorkflowExecutionRecord) -> JsWorkflowResult {
    let output = record
        .step_outputs
        .last()
        .map(|s| s.content.clone())
        .unwrap_or_default();
    let trace_events_json =
        serde_json::to_string(&record.trace_events).unwrap_or_else(|_| "[]".to_string());
    JsWorkflowResult {
        output,
        run_id: record.run_id.to_string(),
        step_count: record.step_outputs.len() as u32,
        trace_events_json,
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn provider_from_js(
    row: Option<JsProviderInput>,
) -> std::result::Result<(Option<Arc<dyn ModelProvider>>, u32, f32), Error> {
    let Some(input) = row else {
        return Ok((None, 0, 0.0));
    };
    let provider_id = match input.kind.as_str() {
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
        model: input.model,
        api_key_env: api_key_env.to_string(),
        params: None,
    };
    let provider = ProviderRuntime::from_config(&config)
        .map_err(|e| configuration_error(format!("[ArcFlow] {e}")))?;
    Ok((Some(provider), input.max_tokens, input.temperature as f32))
}

#[allow(clippy::too_many_arguments)]
fn execute_with_config_sync(
    workflow_name: String,
    workflow_id: String,
    agents: Vec<JsAgentInput>,
    steps: Vec<JsStepInput>,
    run_input: String,
    provider: Option<JsProviderInput>,
    exec_config_json: Option<String>,
    graph_json: Option<String>,
) -> std::result::Result<JsWorkflowResult, Error> {
    let wf_id = Uuid::parse_str(&workflow_id)
        .map_err(|_| configuration_error("Invalid workflow id."))?;
    let (mut workflow, agent_map) = build_workflow(workflow_name, wf_id, &agents, &steps)?;
    if let Some(raw) = graph_json {
        crate::graph::apply_graph_json(&mut workflow, &raw).map_err(configuration_error)?;
    }
    let (provider, max_tokens, temperature) = provider_from_js(provider)?;
    let exec_config = parse_execution_config(exec_config_json.as_deref())
        .map_err(configuration_error)?;
    let engine = WorkflowEngine::new();
    let record = engine
        .execute_with_config(
            &workflow,
            &agent_map,
            &run_input,
            None,
            None,
            provider,
            max_tokens,
            temperature,
            &exec_config,
            None,
        )
        .map_err(workflow_run_error_to_napi)?;
    Ok(record_to_js(record))
}

#[napi(object)]
pub struct JsStreamWorkflowResult {
    pub events_json: String,
    pub output: String,
    pub run_id: String,
    pub step_count: u32,
    pub trace_events_json: String,
}

fn drain_stream_events(
    mut rx: tokio::sync::mpsc::Receiver<StreamEvent>,
) -> Vec<StreamEvent> {
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    events
}

#[allow(clippy::too_many_arguments)]
fn execute_with_config_stream_sync(
    workflow_name: String,
    workflow_id: String,
    agents: Vec<JsAgentInput>,
    steps: Vec<JsStepInput>,
    run_input: String,
    provider: Option<JsProviderInput>,
    exec_config_json: Option<String>,
    graph_json: Option<String>,
) -> std::result::Result<JsStreamWorkflowResult, Error> {
    let wf_id = Uuid::parse_str(&workflow_id)
        .map_err(|_| configuration_error("Invalid workflow id."))?;
    let (mut workflow, agent_map) = build_workflow(workflow_name, wf_id, &agents, &steps)?;
    if let Some(raw) = graph_json {
        crate::graph::apply_graph_json(&mut workflow, &raw).map_err(configuration_error)?;
    }
    let (provider, max_tokens, temperature) = provider_from_js(provider)?;
    let mut exec_config = parse_execution_config(exec_config_json.as_deref())
        .map_err(configuration_error)?;
    exec_config.stream = Some(StreamConfig { enabled: true });
    let (tx, rx) = default_stream_pair();
    let engine = WorkflowEngine::new();
    let record = engine
        .execute_with_config(
            &workflow,
            &agent_map,
            &run_input,
            None,
            None,
            provider,
            max_tokens,
            temperature,
            &exec_config,
            Some(tx),
        )
        .map_err(workflow_run_error_to_napi)?;
    let events = drain_stream_events(rx);
    let events_json =
        serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string());
    let base = record_to_js(record);
    Ok(JsStreamWorkflowResult {
        events_json,
        output: base.output,
        run_id: base.run_id,
        step_count: base.step_count,
        trace_events_json: base.trace_events_json,
    })
}

#[napi]
pub async fn execute_workflow_stream(
    workflow_name: String,
    workflow_id: String,
    agents: Vec<JsAgentInput>,
    steps: Vec<JsStepInput>,
    run_input: String,
    provider: Option<JsProviderInput>,
    exec_config_json: Option<String>,
    graph_json: Option<String>,
) -> Result<JsStreamWorkflowResult> {
    match tokio::task::spawn_blocking(move || {
        execute_with_config_stream_sync(
            workflow_name,
            workflow_id,
            agents,
            steps,
            run_input,
            provider,
            exec_config_json,
            graph_json,
        )
    })
    .await
    {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(Error::from_reason(format!(
            "[ArcFlow] Runtime task failed: {err}"
        ))),
    }
}

#[napi]
pub async fn execute_workflow(
    workflow_name: String,
    workflow_id: String,
    agents: Vec<JsAgentInput>,
    steps: Vec<JsStepInput>,
    run_input: String,
    provider: Option<JsProviderInput>,
    exec_config_json: Option<String>,
    graph_json: Option<String>,
) -> Result<JsWorkflowResult> {
    match tokio::task::spawn_blocking(move || {
        execute_with_config_sync(
            workflow_name,
            workflow_id,
            agents,
            steps,
            run_input,
            provider,
            exec_config_json,
            graph_json,
        )
    })
    .await
    {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(Error::from_reason(format!(
            "[ArcFlow] Runtime task failed: {err}"
        ))),
    }
}

#[napi]
pub async fn execute_resume_workflow(
    workflow_name: String,
    workflow_id: String,
    agents: Vec<JsAgentInput>,
    steps: Vec<JsStepInput>,
    original_run_id: String,
    provider: Option<JsProviderInput>,
    exec_config_json: Option<String>,
) -> Result<JsWorkflowResult> {
    match tokio::task::spawn_blocking(move || {
        let wf_id = Uuid::parse_str(&workflow_id)
            .map_err(|_| configuration_error("Invalid workflow id."))?;
        let (workflow, agent_map) = build_workflow(workflow_name, wf_id, &agents, &steps)?;
        let (provider, max_tokens, temperature) = provider_from_js(provider)?;
        let exec_config = parse_execution_config(exec_config_json.as_deref())
            .map_err(configuration_error)?;
        let engine = WorkflowEngine::new();
        let record = engine
            .resume_with_config(
                &workflow,
                &agent_map,
                &original_run_id,
                None,
                None,
                provider,
                max_tokens,
                temperature,
                &exec_config,
            )
            .map_err(workflow_run_error_to_napi)?;
        Ok(record_to_js(record))
    })
    .await
    {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(Error::from_reason(format!(
            "[ArcFlow] Runtime task failed: {err}"
        ))),
    }
}

#[napi]
pub fn get_execution_trace_json(run_id: String) -> Result<String> {
    match get_execution_trace(&run_id) {
        Some(trace) => serde_json::to_string(&trace).map_err(|e| {
            Error::from_reason(format!("[ArcFlow] Failed to serialize trace: {e}"))
        }),
        None => Err(trace_not_found(&run_id)),
    }
}

#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
