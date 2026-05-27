//! Build RCS types from Python-provided step data.

use std::collections::HashMap;

use arcflow_core::rcs::types::{
    AgentDefinition, MemoryConfig, MemoryScope, MemoryType, StepDefinition, ToolDefinition,
    WorkflowDefinition,
};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use serde_json::Value;
use uuid::Uuid;

use crate::errors::{configuration_error, parse_uuid};

/// One agent registered for the workflow run.
#[derive(Debug, Clone)]
pub struct AgentInput {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub instructions: String,
    pub tools: Vec<ToolInput>,
    pub memory: Option<MemoryInput>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // description reserved until RCS ToolDefinition gains a field
pub struct ToolInput {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // namespace reserved until runtime wires durable backends on agent path
pub struct MemoryInput {
    pub memory_type: MemoryType,
    pub scope: MemoryScope,
    pub namespace: Option<String>,
    pub ttl_seconds: Option<u64>,
}

/// One step in execution order.
#[derive(Debug, Clone)]
pub struct StepInput {
    pub step_id: Uuid,
    pub agent_id: Uuid,
    pub order: u32,
}

fn parse_tool_row(item: Bound<'_, PyAny>) -> PyResult<ToolInput> {
    let tuple = item.downcast::<PyTuple>()?;
    if tuple.len() != 4 {
        return Err(configuration_error(
            "Internal tool tuple must have four fields (name, description, schema_json, timeout).",
        ));
    }
    let schema_json: String = tuple.get_item(2)?.extract()?;
    let schema: Value = serde_json::from_str(&schema_json)
        .map_err(|e| configuration_error(format!("Tool input_schema is not valid JSON: {e}")))?;
    let timeout: f64 = tuple.get_item(3)?.extract()?;
    Ok(ToolInput {
        name: tuple.get_item(0)?.extract()?,
        description: tuple.get_item(1)?.extract()?,
        input_schema: schema,
        timeout_secs: timeout.max(1.0) as u64,
    })
}

fn parse_memory_json(raw: &str) -> PyResult<MemoryInput> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| configuration_error(format!("Invalid memory JSON: {e}")))?;
    let memory_type = match v.get("memory_type").and_then(|x| x.as_str()) {
        Some("Session") => MemoryType::Session,
        Some("Shared") => MemoryType::Shared,
        Some("Persistent") => MemoryType::Persistent,
        Some("Vector") => MemoryType::Vector,
        other => {
            return Err(configuration_error(format!(
                "Unknown memory_type: {:?}",
                other
            )))
        }
    };
    let scope = match v.get("scope").and_then(|x| x.as_str()) {
        Some("Agent") => MemoryScope::Agent,
        Some("Workflow") => MemoryScope::Workflow,
        Some("Global") => MemoryScope::Global,
        other => return Err(configuration_error(format!("Unknown scope: {:?}", other))),
    };
    Ok(MemoryInput {
        memory_type,
        scope,
        namespace: v
            .get("namespace")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        ttl_seconds: v.get("ttl_seconds").and_then(|x| x.as_u64()),
    })
}

pub fn parse_agent_tuple(item: Bound<'_, PyAny>) -> PyResult<AgentInput> {
    let tuple = item.downcast::<PyTuple>()?;
    if tuple.len() != 6 {
        return Err(configuration_error(
            "Internal agent tuple must have six fields (id, name, role, instructions, tools, memory).",
        ));
    }
    let id = parse_uuid("agent_id", tuple.get_item(0)?.extract::<String>()?.as_str())?;
    let tools_list = tuple.get_item(4)?;
    let tools_iter = tools_list.downcast::<PyList>()?;
    let mut tools = Vec::new();
    for t in tools_iter.iter() {
        tools.push(parse_tool_row(t)?);
    }
    let memory: Option<MemoryInput> = match tuple.get_item(5)?.extract::<Option<String>>()? {
        Some(raw) => Some(parse_memory_json(&raw)?),
        None => None,
    };
    Ok(AgentInput {
        id,
        name: tuple.get_item(1)?.extract()?,
        role: tuple.get_item(2)?.extract()?,
        instructions: tuple.get_item(3)?.extract()?,
        tools,
        memory,
    })
}

pub fn parse_step_tuple(item: Bound<'_, PyAny>) -> PyResult<StepInput> {
    let tuple = item.downcast::<PyTuple>()?;
    if tuple.len() != 3 {
        return Err(configuration_error(
            "Internal step tuple must have three fields (step_id, agent_id, order).",
        ));
    }
    let step_id = parse_uuid("step_id", tuple.get_item(0)?.extract::<String>()?.as_str())?;
    let agent_id = parse_uuid("agent_id", tuple.get_item(1)?.extract::<String>()?.as_str())?;
    let order: u32 = tuple.get_item(2)?.extract()?;
    Ok(StepInput {
        step_id,
        agent_id,
        order,
    })
}

pub fn build_workflow(
    workflow_name: String,
    workflow_id: Uuid,
    agents: Vec<AgentInput>,
    steps: Vec<StepInput>,
) -> PyResult<(WorkflowDefinition, HashMap<Uuid, AgentDefinition>)> {
    let mut agent_map = HashMap::new();
    for a in agents {
        let tool_defs: Option<Vec<ToolDefinition>> = if a.tools.is_empty() {
            None
        } else {
            Some(
                a.tools
                    .iter()
                    .map(|t| ToolDefinition {
                        name: t.name.clone(),
                        input_schema: t.input_schema.clone(),
                        permissions: None,
                    })
                    .collect(),
            )
        };
        let memory_config = a.memory.as_ref().map(|m| MemoryConfig {
            memory_type: m.memory_type,
            scope: m.scope,
            ttl_seconds: m.ttl_seconds,
        });
        agent_map.insert(
            a.id,
            AgentDefinition {
                id: a.id,
                name: a.name,
                role: a.role,
                instructions: a.instructions,
                tools: tool_defs,
                memory_config,
            },
        );
    }
    let step_defs: Vec<StepDefinition> = steps
        .into_iter()
        .map(|s| StepDefinition {
            id: s.step_id,
            agent_id: s.agent_id,
            order: s.order,
            fallback_step_id: None,
        })
        .collect();
    let workflow = WorkflowDefinition {
        id: workflow_id,
        name: workflow_name,
        steps: step_defs,
        retry_policy: None,
    };
    Ok((workflow, agent_map))
}

pub fn build_tool_runtime(agents: &[AgentInput]) -> arcflow_core::tools::ToolRuntime {
    use arcflow_core::tools::{RegisteredTool, ToolRuntime};
    let mut runtime = ToolRuntime::new();
    for agent in agents {
        for tool in &agent.tools {
            let _ = runtime.register(RegisteredTool {
                name: tool.name.clone(),
                input_schema: tool.input_schema.clone(),
                timeout_secs: tool.timeout_secs,
            });
        }
    }
    runtime
}
