//! Sequential step execution with state handoff.

use std::collections::HashMap;

use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::RuntimeError;
use crate::rcs::types::{AgentDefinition, WorkflowDefinition};

use super::record::WorkflowExecutionRecord;
use super::run::run_sorted_steps;
use super::validation::validate_workflow;

pub struct WorkflowEngine {
    agent_runtime: AgentRuntime,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            agent_runtime: AgentRuntime::new(),
        }
    }

    pub fn execute(
        &self,
        workflow: &WorkflowDefinition,
        agents: &HashMap<Uuid, AgentDefinition>,
        run_input: &str,
    ) -> Result<WorkflowExecutionRecord, RuntimeError> {
        validate_workflow(workflow, agents)?;
        run_sorted_steps(&self.agent_runtime, workflow, agents, run_input)
    }
}
