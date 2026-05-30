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
