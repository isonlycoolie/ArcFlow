//! Graph workflow execution loop (Phase 1.1).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use uuid::Uuid;

use crate::agent::AgentRuntime;
use crate::error::RuntimeError;
use crate::memory::MemoryCoordinator;
use crate::providers::ModelProvider;
use crate::rcs::types::{AgentDefinition, JoinNode, StepDefinition, WorkflowDefinition};
use crate::retry::RetryConfig;
use crate::state::StateEngine;
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::tracing::{
    emitter::TraceEmitter, events::TraceEventKind, otel_export::maybe_export_trace,
    sprint5_emitter::TraceEventEmitter, with_store, TokenUsage,
};
use crate::workflow::run::{check_workflow_timeout, partial_record, run_one_step, RunLoop};

use super::checkpoint::persist_graph_checkpoint;
use super::executor::GraphExecutor;
use crate::workflow::{ExecutionConfig, WorkflowExecutionRecord, WorkflowRunError};

struct JoinGate {
    defs: HashMap<String, JoinNode>,
    completed: HashSet<String>,
    executed: HashSet<String>,
}

impl JoinGate {
    fn new(joins: &[JoinNode]) -> Self {
        Self {
            defs: joins.iter().map(|j| (j.id.clone(), j.clone())).collect(),
            completed: HashSet::new(),
            executed: HashSet::new(),
        }
    }

    fn mark_completed(&mut self, node_id: &str) {
        self.completed.insert(node_id.to_string());
    }

    fn is_join(&self, node_id: &str) -> bool {
        self.defs.contains_key(node_id)
    }

    fn is_ready(&self, join_id: &str) -> bool {
        self.defs
            .get(join_id)
            .is_some_and(|j| j.wait_for.iter().all(|b| self.completed.contains(b)))
    }

    fn enqueue_ready_joins(&mut self, pending: &mut Vec<String>) {
        for join_id in self.defs.keys().cloned().collect::<Vec<_>>() {
            if self.executed.contains(&join_id) {
                continue;
            }
            if self.is_ready(&join_id) {
                self.executed.insert(join_id.clone());
                pending.push(join_id);
            }
        }
    }

    fn enqueue_targets(&mut self, pending: &mut Vec<String>, targets: Vec<String>) {
        for target in targets {
            if self.is_join(&target) {
                if self.is_ready(&target) && !self.executed.contains(&target) {
                    self.executed.insert(target.clone());
                    pending.push(target);
                }
            } else {
                pending.push(target);
            }
        }
    }
}

/// Executes a graph-mode workflow using the same per-step path as linear runs.
#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
pub fn run_graph_loop(
    agent_runtime: &AgentRuntime,
    workflow: &WorkflowDefinition,
    agents: &HashMap<Uuid, AgentDefinition>,
    run_input: &str,
    tool_runtime: Option<&ToolRuntime>,
    tool_invoker: Option<Arc<dyn ToolInvoker>>,
    provider: Option<Arc<dyn ModelProvider>>,
    provider_max_tokens: u32,
    provider_temperature: f32,
    exec_config: &ExecutionConfig,
