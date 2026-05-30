//! Live workflow stream iterator for the TypeScript SDK.

use std::sync::Mutex;

use arcflow_core::streaming::{StreamEvent, StreamRunBridge};
use arcflow_core::workflow::{
    StreamConfig, WorkflowEngine, WorkflowExecutionRecord, WorkflowRunError,
};
use napi::bindgen_prelude::Result;
use napi::Error;
use napi_derive::napi;
use uuid::Uuid;

use crate::errors::{configuration_error, workflow_run_error_to_napi};
use crate::execution_config::parse_execution_config;
use crate::types::{build_workflow, JsAgentInput, JsStepInput};
use crate::workflow::{provider_from_js, record_to_js, JsProviderInput, JsWorkflowResult};

type RunResult = std::result::Result<WorkflowExecutionRecord, WorkflowRunError>;

fn stream_event_to_json(event: StreamEvent) -> Result<String> {
    serde_json::to_string(&event).map_err(|e| {
        Error::from_reason(format!("[ArcFlow] Failed to serialize stream event: {e}"))
    })
}

#[napi]
pub struct JsWorkflowStreamIterator {
    bridge: Mutex<Option<StreamRunBridge<RunResult>>>,
    events_exhausted: Mutex<bool>,
}

#[napi]
impl JsWorkflowStreamIterator {
    #[napi]
    pub fn poll_event(&self) -> Result<Option<String>> {
        let mut exhausted = self
            .events_exhausted
            .lock()
            .map_err(|_| Error::from_reason("[ArcFlow] Stream lock poisoned."))?;
        if *exhausted {
            return Ok(None);
        }
        let mut guard = self
            .bridge
            .lock()
            .map_err(|_| Error::from_reason("[ArcFlow] Stream lock poisoned."))?;
        let bridge = guard.as_mut().ok_or_else(|| {
            configuration_error("Stream iterator already finalized.")
        })?;
        match bridge.recv_event() {
            Some(event) => stream_event_to_json(event).map(Some),
            None => {
                *exhausted = true;
                Ok(None)
            }
        }
    }

    #[napi]
    pub fn finalize(&self) -> Result<JsWorkflowResult> {
        let mut guard = self
            .bridge
            .lock()
            .map_err(|_| Error::from_reason("[ArcFlow] Stream lock poisoned."))?;
        let bridge = guard.take().ok_or_else(|| {
            configuration_error("Stream iterator already finalized.")
        })?;
        let mut exhausted = self
            .events_exhausted
            .lock()
            .map_err(|_| Error::from_reason("[ArcFlow] Stream lock poisoned."))?;
        *exhausted = true;
        match bridge.join().result {
            Ok(record) => Ok(record_to_js(record)),
            Err(err) => Err(workflow_run_error_to_napi(err)),
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[napi]
pub fn start_workflow_stream(
    workflow_name: String,
    workflow_id: String,
    agents: Vec<JsAgentInput>,
    steps: Vec<JsStepInput>,
    run_input: String,
    provider: Option<JsProviderInput>,
    exec_config_json: Option<String>,
    graph_json: Option<String>,
) -> Result<JsWorkflowStreamIterator> {
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
    let engine = WorkflowEngine::new();
    let bridge = StreamRunBridge::spawn(move |stream_tx| {
        engine.execute_with_config(
            &workflow,
            &agent_map,
            &run_input,
            None,
            None,
            provider,
            max_tokens,
            temperature,
            &exec_config,
            Some(stream_tx),
        )
    });
    Ok(JsWorkflowStreamIterator {
        bridge: Mutex::new(Some(bridge)),
        events_exhausted: Mutex::new(false),
    })
}
