//! Build RCS types from JS-provided workflow data.

use std::collections::HashMap;

use arcflow_core::rcs::types::{
    AgentDefinition, HitlConfig, StepDefinition, WorkflowDefinition,
};
use napi::Error;
use napi_derive::napi;
use uuid::Uuid;

use crate::errors::parse_uuid;

#[napi(object)]
pub struct JsAgentInput {
    pub id: String,
    pub name: String,
    pub role: String,
    pub instructions: String,
}

#[napi(object)]
pub struct JsStepInput {
    pub step_id: String,
    pub agent_id: String,
    pub order: u32,
    pub hitl_json: Option<String>,
}

pub fn build_workflow(
    workflow_name: String,
    workflow_id: Uuid,
    agents: &[JsAgentInput],
    steps: &[JsStepInput],
) -> std::result::Result<(WorkflowDefinition, HashMap<Uuid, AgentDefinition>), Error> {
    let mut agent_map = HashMap::new();
    for agent in agents {
        let id = parse_uuid("agent_id", &agent.id)?;
        agent_map.insert(
            id,
            AgentDefinition {
                id,
                name: agent.name.clone(),
                role: agent.role.clone(),
                instructions: agent.instructions.clone(),
                tools: None,
                memory_config: None,
                context: None,
                tool_execution: None,
            },
        );
    }
    let mut step_defs = Vec::new();
    for step in steps {
        let step_id = parse_uuid("step_id", &step.step_id)?;
        let agent_id = parse_uuid("agent_id", &step.agent_id)?;
        let hitl = match step.hitl_json.as_deref() {
            Some(raw) if !raw.is_empty() => Some(parse_hitl_json(raw)?),
            _ => None,
        };
        step_defs.push(StepDefinition {
            id: step_id,
            agent_id,
            order: step.order,
            fallback_step_id: None,
            hitl,
        });
    }
    let workflow = WorkflowDefinition {
        id: workflow_id,
        name: workflow_name,
        steps: step_defs,
        retry_policy: None,
        execution_mode: arcflow_core::rcs::types::ExecutionMode::Linear,
        graph: None,
        external_bindings: None,
    };
    Ok((workflow, agent_map))
}

fn parse_hitl_json(raw: &str) -> Result<HitlConfig, Error> {
    let v: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| Error::from_reason(format!("[ArcFlow] Invalid HITL JSON: {e}")))?;
    let approval_key = v
        .get("approval_key")
        .and_then(|x| x.as_str())
        .ok_or_else(|| Error::from_reason("[ArcFlow] HITL config requires approval_key"))?
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
