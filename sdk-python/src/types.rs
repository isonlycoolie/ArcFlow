//! Build RCS types from Python-provided step data.

use std::collections::HashMap;

use arcflow_core::rcs::types::{
    AgentDefinition, HitlConfig, MemoryChunkingConfig, MemoryConfig, MemoryRetrievalConfig,
    MemoryScope, MemoryType, RerankProviderSpec, RetrievalModeSpec, StepDefinition,
    ToolDefinition, WorkflowDefinition,
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
    pub embedding: Option<String>,
    pub retrieval: Option<MemoryRetrievalConfig>,
    pub chunking: Option<MemoryChunkingConfig>,
}

/// One step in execution order.
#[derive(Debug, Clone)]
pub struct StepInput {
    pub step_id: Uuid,
    pub agent_id: Uuid,
    pub order: u32,
    pub fallback_step_id: Option<Uuid>,
    pub hitl_json: Option<String>,
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
        embedding: v
            .get("embedding")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        retrieval: v.get("retrieval").and_then(parse_retrieval_json),
        chunking: v.get("chunking").and_then(parse_chunking_json),
    })
}

fn parse_retrieval_json(v: &Value) -> Option<MemoryRetrievalConfig> {
    let mode = match v.get("mode").and_then(|x| x.as_str()).unwrap_or("dense") {
        "dense" => RetrievalModeSpec::Dense,
        "hybrid" => RetrievalModeSpec::Hybrid,
        _ => return None,
    };
    let rerank = v.get("rerank").and_then(|x| x.as_str()).and_then(|s| match s {
        "cohere" => Some(RerankProviderSpec::Cohere),
        "local" => Some(RerankProviderSpec::Local),
        _ => None,
    });
    Some(MemoryRetrievalConfig {
        mode,
        dense_weight: v.get("dense_weight").and_then(|x| x.as_f64()).unwrap_or(0.7) as f32,
        sparse_weight: v.get("sparse_weight").and_then(|x| x.as_f64()).unwrap_or(0.3) as f32,
        rerank,
        top_k: v.get("top_k").and_then(|x| x.as_u64()).map(|n| n as u32),
    })
}

fn parse_chunking_json(v: &Value) -> Option<MemoryChunkingConfig> {
    Some(MemoryChunkingConfig {
        strategy: v
            .get("strategy")
            .and_then(|x| x.as_str())
            .unwrap_or("recursive")
            .to_string(),
        chunk_size: v.get("chunk_size").and_then(|x| x.as_u64()).unwrap_or(512) as usize,
        overlap: v.get("overlap").and_then(|x| x.as_u64()).unwrap_or(64) as usize,
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
    if tuple.len() < 3 || tuple.len() > 5 {
        return Err(configuration_error(
            "Internal step tuple must have 3–5 fields (step_id, agent_id, order, fallback_step_id?, hitl_json?).",
        ));
    }
    let step_id = parse_uuid("step_id", tuple.get_item(0)?.extract::<String>()?.as_str())?;
    let agent_id = parse_uuid("agent_id", tuple.get_item(1)?.extract::<String>()?.as_str())?;
    let order: u32 = tuple.get_item(2)?.extract()?;
    let fallback_step_id = if tuple.len() >= 4 {
        match tuple.get_item(3)?.extract::<Option<String>>()? {
            Some(raw) if !raw.is_empty() => Some(parse_uuid("fallback_step_id", raw.as_str())?),
            _ => None,
        }
    } else {
        None
    };
    let hitl_json = if tuple.len() == 5 {
        tuple.get_item(4)?.extract::<Option<String>>()?
    } else {
        None
    };
    Ok(StepInput {
        step_id,
        agent_id,
        order,
        fallback_step_id,
        hitl_json,
    })
}

fn parse_hitl_json(raw: &str) -> PyResult<HitlConfig> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| configuration_error(format!("Invalid HITL JSON: {e}")))?;
    let approval_key = v
        .get("approval_key")
        .and_then(|x| x.as_str())
        .ok_or_else(|| configuration_error("HITL config requires approval_key"))?
        .to_string();
    let timeout_seconds = v
        .get("timeout_seconds")
        .and_then(|x| x.as_u64())
        .unwrap_or(3600);
    let interrupt = v
        .get("interrupt")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);
    Ok(HitlConfig {
        approval_key,
        timeout_seconds,
        interrupt,
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
            namespace: m.namespace.clone(),
            ttl_seconds: m.ttl_seconds,
            embedding: m.embedding.clone(),
            retrieval: m.retrieval.clone(),
            chunking: m.chunking.clone(),
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
        .map(|s| {
            let hitl = match s.hitl_json.as_deref() {
                Some(raw) if !raw.is_empty() => Some(parse_hitl_json(raw)?),
                _ => None,
            };
            Ok(StepDefinition {
                id: s.step_id,
                agent_id: s.agent_id,
                order: s.order,
                fallback_step_id: s.fallback_step_id,
                hitl,
            })
        })
        .collect::<PyResult<Vec<_>>>()?;
    let workflow = WorkflowDefinition {
        id: workflow_id,
        name: workflow_name,
        steps: step_defs,
        retry_policy: None,
        execution_mode: arcflow_core::rcs::types::ExecutionMode::Linear,
        graph: None,
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
