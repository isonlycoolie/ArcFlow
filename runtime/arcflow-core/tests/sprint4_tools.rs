//! Sprint 4 tool execution through workflow engine.

use std::collections::HashMap;
use std::sync::Arc;

use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionStatus, MemoryConfig, MemoryScope, MemoryType, StepDefinition,
    ToolDefinition, TraceEventKind, WorkflowDefinition,
};
use arcflow_core::tools::{RegisteredTool, ToolError, ToolInvoker, ToolRuntime};
use arcflow_core::workflow::WorkflowEngine;
use serde_json::{json, Value};
use uuid::Uuid;

struct EchoInvoker;

impl ToolInvoker for EchoInvoker {
    fn invoke(&self, _name: &str, input: &Value) -> Result<String, ToolError> {
        Ok(input
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string())
    }
}

#[test]
fn workflow_runs_agent_with_tool() {
    let aid = Uuid::new_v4();
    let sid = Uuid::new_v4();
    let mut agents = HashMap::new();
    agents.insert(
        aid,
        AgentDefinition {
            id: aid,
            name: "a".into(),
            role: "researcher".into(),
            instructions: "i".into(),
            tools: Some(vec![ToolDefinition {
                name: "echo".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": { "message": { "type": "string" } }
                }),
                permissions: None,
            }]),
            memory_config: None,
        },
    );
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "t".into(),
        steps: vec![StepDefinition {
            id: sid,
            agent_id: aid,
            order: 1,
            fallback_step_id: None,
        }],
        retry_policy: None,
    };
    let mut runtime = ToolRuntime::new();
    runtime
        .register(RegisteredTool {
            name: "echo".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "message": { "type": "string" } }
            }),
            timeout_secs: 5,
        })
        .unwrap();
    let invoker = Arc::new(EchoInvoker);
    let record = WorkflowEngine::new()
        .execute_with_tools(&wf, &agents, "hi", Some(&runtime), Some(invoker), None, 0, 0.0)
        .unwrap();
    assert_eq!(record.step_outputs.len(), 1);
    assert_eq!(record.step_outputs[0].status, ExecutionStatus::Completed);
}

#[test]
fn workflow_with_session_memory_records_trace_events() {
    let aid = Uuid::new_v4();
    let sid = Uuid::new_v4();
    let mut agents = HashMap::new();
    agents.insert(
        aid,
        AgentDefinition {
            id: aid,
            name: "mem".into(),
            role: "researcher".into(),
            instructions: "i".into(),
            tools: None,
            memory_config: Some(MemoryConfig {
                memory_type: MemoryType::Session,
                scope: MemoryScope::Agent,
                namespace: None,
                ttl_seconds: None,
            }),
        },
    );
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "mem-wf".into(),
        steps: vec![StepDefinition {
            id: sid,
            agent_id: aid,
            order: 1,
            fallback_step_id: None,
        }],
        retry_policy: None,
    };
    let record = WorkflowEngine::new()
        .execute_with_tools(&wf, &agents, "remember", None, None, None, 0, 0.0)
        .unwrap();
    assert!(record.trace_events.iter().any(|e| {
        matches!(
            e.event_kind,
            TraceEventKind::MemoryWrite | TraceEventKind::MemoryRead
        )
    }));
}
