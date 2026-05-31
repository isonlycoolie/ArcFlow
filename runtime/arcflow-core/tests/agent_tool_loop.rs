//! LLM tool loop integration tests (Phase 2-Pro).

use std::collections::HashMap;
use std::sync::Arc;

use arcflow_core::providers::mock::MockToolProvider;
use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionMode, StepDefinition, ToolDefinition, ToolExecutionConfig,
    ToolExecutionMode, WorkflowDefinition,
};
use arcflow_core::tools::{RegisteredTool, ToolError, ToolInvoker, ToolRuntime};
use arcflow_core::workflow::WorkflowEngine;
use serde_json::{json, Value};
use uuid::Uuid;

struct QueryInvoker;

impl ToolInvoker for QueryInvoker {
    fn invoke(&self, _name: &str, input: &Value) -> Result<String, ToolError> {
        let q = input
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        Ok(format!("results for {q}"))
    }
}

fn tool_agent(id: Uuid, tools: Vec<ToolDefinition>) -> AgentDefinition {
    AgentDefinition {
        id,
        name: "tool-agent".into(),
        role: "researcher".into(),
        instructions: "Use tools when needed.".into(),
        tools: Some(tools),
        memory_config: None,
        context: None,
        tool_execution: Some(ToolExecutionConfig {
            mode: ToolExecutionMode::LlmSelect,
            max_iterations: 5,
        }),
    }
}

#[test]
fn llm_selects_tool_and_result_in_final_output() {
    let aid = Uuid::new_v4();
    let sid = Uuid::new_v4();
    let tools = vec![
        ToolDefinition {
            name: "web_search".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "query": { "type": "string" } }
            }),
            permissions: None,
        },
        ToolDefinition {
            name: "calc".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "expr": { "type": "string" } }
            }),
            permissions: None,
        },
        ToolDefinition {
            name: "noop".into(),
            input_schema: json!({ "type": "object" }),
            permissions: None,
        },
    ];
    let mut agents = HashMap::new();
    agents.insert(aid, tool_agent(aid, tools.clone()));
    let wf = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "tool-loop".into(),
        steps: vec![StepDefinition {
            id: sid,
            agent_id: aid,
            order: 1,
            fallback_step_id: None,
            hitl: None,
        }],
        retry_policy: None,
        execution_mode: ExecutionMode::Linear,
        graph: None,
    };
    let mut tool_runtime = ToolRuntime::new();
    for t in &tools {
        tool_runtime
            .register(RegisteredTool {
                name: t.name.clone(),
                input_schema: t.input_schema.clone(),
                timeout_secs: 5,
            })
            .unwrap();
    }
    let provider = Arc::new(MockToolProvider::new());
    let invoker = Arc::new(QueryInvoker);
    let record = WorkflowEngine::new()
        .execute_with_tools(
            &wf,
            &agents,
            "analyze AAPL",
            Some(&tool_runtime),
            Some(invoker),
            Some(provider),
            256,
            0.0,
        )
        .unwrap();
    let out = &record.step_outputs[0].content;
    assert!(out.contains("final answer using"));
    assert!(out.contains("results for test"));
}
