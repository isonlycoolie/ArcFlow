//! Sequential step execution with state handoff.

use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::providers::{default_max_tokens, default_temperature, ModelProvider};
use crate::rcs::types::{AgentDefinition, ExecutionMode, WorkflowDefinition};
use crate::tools::{ToolInvoker, ToolRuntime};

use super::record::WorkflowExecutionRecord;
use super::execution_config::ExecutionConfig;
use super::graph::run_graph_loop;
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
            &ExecutionConfig::default(),
            None,
            None,
        )
    }

    /// Executes with optional tool runtime, invoker, and LLM provider (Sprint 6).
    #[allow(clippy::result_large_err)]
    #[allow(clippy::too_many_arguments)]
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
            &ExecutionConfig::default(),
            None,
            None,
        )
    }

    /// Executes with Sprint 7 retry, timeout, and recovery options.
    #[allow(clippy::result_large_err)]
    #[allow(clippy::too_many_arguments)]
    pub fn execute_with_config(
        &self,
        workflow: &WorkflowDefinition,
        agents: &HashMap<Uuid, AgentDefinition>,
        run_input: &str,
        tool_runtime: Option<&ToolRuntime>,
        tool_invoker: Option<Arc<dyn ToolInvoker>>,
        provider: Option<Arc<dyn ModelProvider>>,
        provider_max_tokens: u32,
        provider_temperature: f32,
        exec_config: &ExecutionConfig,
        stream_tx: Option<crate::streaming::StreamChannelSender>,
    ) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
        validate_workflow(workflow, agents)?;
        match workflow.execution_mode {
            ExecutionMode::Graph => run_graph_loop(
                &self.agent_runtime,
                workflow,
                agents,
                run_input,
                tool_runtime,
                tool_invoker,
                provider,
                provider_max_tokens,
                provider_temperature,
                exec_config,
                stream_tx,
            ),
            ExecutionMode::Linear => run_sorted_steps(
                &self.agent_runtime,
                workflow,
                agents,
                run_input,
                tool_runtime,
                tool_invoker,
                provider,
                provider_max_tokens,
                provider_temperature,
                exec_config,
                None,
                stream_tx,
            ),
        }
    }

    /// Resumes a failed run from PostgreSQL recovery state (Sprint 7).
    #[allow(clippy::result_large_err)]
    #[allow(clippy::too_many_arguments)]
    pub fn resume_with_config(
        &self,
        workflow: &WorkflowDefinition,
        agents: &HashMap<Uuid, AgentDefinition>,
        original_run_id: &str,
        tool_runtime: Option<&ToolRuntime>,
        tool_invoker: Option<Arc<dyn ToolInvoker>>,
        provider: Option<Arc<dyn ModelProvider>>,
        provider_max_tokens: u32,
        provider_temperature: f32,
        exec_config: &ExecutionConfig,
    ) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
        validate_workflow(workflow, agents)?;
        crate::recovery::resume_workflow(
            &self.agent_runtime,
            workflow,
            agents,
            original_run_id,
            tool_runtime,
            tool_invoker,
            provider,
            provider_max_tokens,
            provider_temperature,
            exec_config,
        )
    }

    /// Resumes a human-interrupted run after approval (Phase 1.4 HITL).
    #[allow(clippy::result_large_err)]
    #[allow(clippy::too_many_arguments)]
    pub fn resume_with_approval(
        &self,
        workflow: &WorkflowDefinition,
        agents: &HashMap<Uuid, AgentDefinition>,
        original_run_id: &str,
        approval_key: &str,
        approval: crate::human::ApprovalResult,
        tool_runtime: Option<&ToolRuntime>,
        tool_invoker: Option<Arc<dyn ToolInvoker>>,
        provider: Option<Arc<dyn ModelProvider>>,
        provider_max_tokens: u32,
        provider_temperature: f32,
        exec_config: &ExecutionConfig,
        resolve_in_db: bool,
    ) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
        validate_workflow(workflow, agents)?;
        crate::human::resume_workflow_with_approval(
            &self.agent_runtime,
            workflow,
            agents,
            original_run_id,
            approval_key,
            approval,
            tool_runtime,
            tool_invoker,
            provider,
            provider_max_tokens,
            provider_temperature,
            exec_config,
            resolve_in_db,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::WorkflowEngine;
    use crate::error::RuntimeError;
    use crate::rcs::types::{AgentDefinition, ExecutionMode, StepDefinition, WorkflowDefinition};
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
            execution_mode: ExecutionMode::Linear,
            graph: None,
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
                hitl: None,
            }],
            retry_policy: None,
            execution_mode: ExecutionMode::Linear,
            graph: None,
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
                hitl: None,
                },
                StepDefinition {
                    id: dup,
                    agent_id: aid,
                    order: 2,
                    fallback_step_id: None,
                hitl: None,
                },
            ],
            retry_policy: None,
            execution_mode: ExecutionMode::Linear,
            graph: None,
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
                hitl: None,
            }],
            retry_policy: None,
            execution_mode: ExecutionMode::Linear,
            graph: None,
        };
        let err = WorkflowEngine::new().execute(&wf, &m, "in").unwrap_err();
        assert!(matches!(
            err,
            WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition { .. })
        ));
    }
}
