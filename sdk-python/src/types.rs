//! Build RCS types from Python-provided step data.

use std::collections::HashMap;

use arcflow_core::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use uuid::Uuid;

use crate::errors::{configuration_error, parse_uuid};

/// One agent registered for the workflow run.
#[derive(Debug, Clone)]
pub struct AgentInput {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub instructions: String,
}

/// One step in execution order.
#[derive(Debug, Clone)]
pub struct StepInput {
    pub step_id: Uuid,
    pub agent_id: Uuid,
    pub order: u32,
}

pub fn parse_agent_tuple(item: Bound<'_, PyAny>) -> PyResult<AgentInput> {
    let tuple = item.downcast::<PyTuple>()?;
    if tuple.len() != 4 {
        return Err(configuration_error(
            "Internal agent tuple must have four fields (id, name, role, instructions).",
        ));
    }
    let id = parse_uuid("agent_id", tuple.get_item(0)?.extract::<String>()?.as_str())?;
    Ok(AgentInput {
        id,
        name: tuple.get_item(1)?.extract()?,
        role: tuple.get_item(2)?.extract()?,
        instructions: tuple.get_item(3)?.extract()?,
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
        agent_map.insert(
            a.id,
            AgentDefinition {
                id: a.id,
                name: a.name,
                role: a.role,
                instructions: a.instructions,
                tools: None,
                memory_config: None,
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
