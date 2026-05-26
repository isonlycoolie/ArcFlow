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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::WorkflowEngine;
    use crate::error::RuntimeError;
    use crate::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};

    fn agent(id: Uuid) -> AgentDefinition {
        AgentDefinition {
            id,
            name: "a".into(),
            role: "r".into(),
            instructions: "i".into(),
            tools: None,
            memory_config: None,
        }
    }

    #[test]
    fn execute_empty_step_list_returns_invalid_definition_error() {
        let wf = WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "w".into(),
            steps: vec![],
            retry_policy: None,
        };
        let err = WorkflowEngine::new()
            .execute(&wf, &HashMap::new(), "in")
            .unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::InvalidWorkflowDefinition { .. }
        ));
    }

    #[test]
    fn execute_with_unknown_agent_id_returns_agent_not_found_error() {
        let aid = Uuid::new_v4();
        let sid = Uuid::new_v4();
        let wf = WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "w".into(),
            steps: vec![StepDefinition {
                id: sid,
                agent_id: aid,
                order: 1,
                fallback_step_id: None,
            }],
            retry_policy: None,
        };
        let err = WorkflowEngine::new()
            .execute(&wf, &HashMap::new(), "in")
            .unwrap_err();
        assert!(matches!(err, RuntimeError::AgentNotFound { .. }));
    }

    #[test]
    fn execute_with_duplicate_step_ids_returns_invalid_definition_error() {
        let aid = Uuid::new_v4();
        let dup = Uuid::new_v4();
        let mut m = HashMap::new();
        m.insert(aid, agent(aid));
        let wf = WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "w".into(),
            steps: vec![
                StepDefinition {
                    id: dup,
                    agent_id: aid,
                    order: 1,
                    fallback_step_id: None,
                },
                StepDefinition {
                    id: dup,
                    agent_id: aid,
                    order: 2,
                    fallback_step_id: None,
                },
            ],
            retry_policy: None,
        };
        let err = WorkflowEngine::new().execute(&wf, &m, "in").unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::InvalidWorkflowDefinition { .. }
        ));
    }
}
