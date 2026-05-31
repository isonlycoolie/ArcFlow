//! ArcFlow WASM edge runtime (alpha).
//!
//! Stub linear workflow execution for Cloudflare Workers and similar hosts.
//! Full `arcflow-core` linkage is deferred until native deps are wasm-gated.

pub mod types;
pub mod runner;

use wasm_bindgen::prelude::*;

use crate::runner::run_linear_stub;
use crate::types::{parse_bundle, parse_input};

/// Host-callable JSON API (used by tests and the wasm-bindgen export).
pub fn execute_workflow_json(workflow_json: &str, input_json: &str) -> Result<String, String> {
    let bundle = parse_bundle(workflow_json).map_err(|e| e.message())?;
    let input = parse_input(input_json).map_err(|e| e.message())?;
    let result = run_linear_stub(&bundle, &input).map_err(|e| e.message())?;
    serde_json::to_string(&result).map_err(|e| format!("result serialization failed: {e}"))
}

#[wasm_bindgen(js_name = executeWorkflow)]
pub fn execute_workflow(workflow_json: &str, input_json: &str) -> Result<String, JsValue> {
    execute_workflow_json(workflow_json, input_json).map_err(js_error)
}

fn js_error(message: String) -> JsValue {
    let code = if message.contains("not supported on edge") {
        "unsupported_mode"
    } else if message.contains("invalid workflow JSON") {
        "invalid_json"
    } else if message.contains("agent") && message.contains("not found") {
        "missing_agent"
    } else if message.contains("no steps") {
        "empty_workflow"
    } else {
        "execution_failed"
    };
    JsValue::from_str(&format!("{{\"code\":\"{code}\",\"message\":\"{message}\"}}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_workflow_json_round_trip() {
        let wf_id = uuid::Uuid::new_v4();
        let step_id = uuid::Uuid::new_v4();
        let agent_id = uuid::Uuid::new_v4();
        let workflow = serde_json::json!({
            "workflow": {
                "id": wf_id,
                "name": "echo",
                "execution_mode": "Linear",
                "steps": [{ "id": step_id, "agent_id": agent_id, "order": 0 }]
            },
            "agents": [{
                "id": agent_id,
                "name": "echo",
                "role": "assistant",
                "instructions": "Echo input."
            }]
        });
        let out = execute_workflow_json(&workflow.to_string(), "\"hi\"").unwrap();
        assert!(out.contains("hi"));
    }
}
