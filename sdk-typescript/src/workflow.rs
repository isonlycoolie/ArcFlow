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

fn record_to_js(record: WorkflowExecutionRecord) -> JsWorkflowResult {
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
fn provider_from_js(
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
