//! Build RCS types from JS-provided workflow data.

use std::collections::HashMap;

use arcflow_core::rcs::types::{
    AgentDefinition, StepDefinition, WorkflowDefinition,
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
            },
        );
    }
    let mut step_defs = Vec::new();
    for step in steps {
        let step_id = parse_uuid("step_id", &step.step_id)?;
        let agent_id = parse_uuid("agent_id", &step.agent_id)?;
        step_defs.push(StepDefinition {
            id: step_id,
            agent_id,
            order: step.order,
            fallback_step_id: None,
            hitl: None,
        });
    }
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
