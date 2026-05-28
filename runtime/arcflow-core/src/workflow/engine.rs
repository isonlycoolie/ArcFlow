//! Sequential step execution with state handoff.

use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::providers::{default_max_tokens, default_temperature, ModelProvider};
use crate::rcs::types::{AgentDefinition, WorkflowDefinition};
use crate::tools::{ToolInvoker, ToolRuntime};

use super::record::WorkflowExecutionRecord;
use super::run::run_sorted_steps;
use super::run_error::WorkflowRunError;
use super::validation::validate_workflow;

/// Runs a workflow definition sequentially with state handoff between steps.
pub struct WorkflowEngine {
    agent_runtime: AgentRuntime,
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowEngine {
    /// Builds an engine with the default deterministic stub [`AgentRuntime`].
    pub fn new() -> Self {
        Self {
            agent_runtime: AgentRuntime::new(),
        }
    }

    /// Validates the definition, then runs steps in ascending `order`.
    ///
    /// On agent failure after one or more steps complete, returns [`WorkflowRunError::Failed`]
    /// with the partial [`WorkflowExecutionRecord`].
    #[allow(clippy::result_large_err)] // `Failed` carries partial record by design (ADR-002)
    pub fn execute(
        &self,
        workflow: &WorkflowDefinition,
        agents: &HashMap<Uuid, AgentDefinition>,
        run_input: &str,
    ) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
        validate_workflow(workflow, agents)?;
        run_sorted_steps(
            &self.agent_runtime,
            workflow,
            agents,
            run_input,
            None,
            None,
            None,
            default_max_tokens(),
            default_temperature(),
        )
    }

    /// Executes with optional tool runtime, invoker, and LLM provider (Sprint 6).
    #[allow(clippy::result_large_err)]
    pub fn execute_with_tools(
        &self,
        workflow: &WorkflowDefinition,
        agents: &HashMap<Uuid, AgentDefinition>,
        run_input: &str,
        tool_runtime: Option<&ToolRuntime>,
        tool_invoker: Option<Arc<dyn ToolInvoker>>,
        provider: Option<Arc<dyn ModelProvider>>,
        provider_max_tokens: u32,
        provider_temperature: f32,
    ) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
        validate_workflow(workflow, agents)?;
        run_sorted_steps(
            &self.agent_runtime,
            workflow,
            agents,
            run_input,
            tool_runtime,
            tool_invoker,
            provider,
            provider_max_tokens,
            provider_temperature,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::WorkflowEngine;
    use crate::error::RuntimeError;
    use crate::rcs::types::{AgentDefinition, StepDefinition, WorkflowDefinition};
    use crate::workflow::WorkflowRunError;

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
            WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition { .. })
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
        assert!(matches!(
            err,
            WorkflowRunError::Aborted(RuntimeError::AgentNotFound { .. })
        ));
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
            WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition { .. })
        ));
    }

    #[test]
    fn execute_with_empty_workflow_name_returns_invalid_definition_error() {
        let aid = Uuid::new_v4();
        let sid = Uuid::new_v4();
        let mut m = HashMap::new();
        m.insert(aid, agent(aid));
        let wf = WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "   ".into(),
            steps: vec![StepDefinition {
                id: sid,
                agent_id: aid,
                order: 1,
                fallback_step_id: None,
            }],
            retry_policy: None,
        };
        let err = WorkflowEngine::new().execute(&wf, &m, "in").unwrap_err();
        assert!(matches!(
            err,
            WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition { .. })
        ));
    }
}
