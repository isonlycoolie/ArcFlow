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
) -> Result<WorkflowExecutionRecord, WorkflowRunError> {
    let graph = workflow.graph.as_ref().ok_or_else(|| {
        WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
            reason: "graph block missing for graph execution_mode".into(),
        })
    })?;

    let retry_config = exec_config
        .retry
        .clone()
        .or_else(|| workflow.retry_policy.as_ref().map(RetryConfig::from_rcs));
    if let Some(ref r) = retry_config {
        r.validate().map_err(WorkflowRunError::Aborted)?;
    }
    exec_config.validate().map_err(WorkflowRunError::Aborted)?;

    let step_timeout = exec_config.timeouts.step_timeout;
    let workflow_timeout = exec_config.timeouts.workflow_timeout;
    let run_id = exec_config.run_id.unwrap_or_else(Uuid::new_v4);
    let run_key = run_id.to_string();
    let workflow_started = Instant::now();
    let workflow_deadline = workflow_timeout.map(|wt| workflow_started + wt);

    let step_by_id: HashMap<Uuid, &StepDefinition> =
        workflow.steps.iter().map(|s| (s.id, s)).collect();
    let node_to_step: HashMap<&str, &StepDefinition> = graph
        .nodes
        .iter()
        .filter_map(|n| step_by_id.get(&n.step_ref).map(|s| (n.id.as_str(), *s)))
        .collect();

    let mut executor = GraphExecutor::new(graph.clone()).with_parallel();
    let mut join_gate = JoinGate::new(&graph.join_nodes);
    let mut pending: Vec<String> = vec![graph.entry_node.clone()];
    let mut graph_step_index = 0usize;

    with_store(|store| {
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), store);
        let mut legacy = TraceEmitter::new(run_id);
        sprint5.emit(TraceEventKind::WorkflowStarted {
            run_id: run_key.clone(),
            workflow_name: workflow.name.clone(),
            step_count: graph.nodes.len(),
        });

        let mut state = StateEngine::new();
        let mut memory = MemoryCoordinator::new(run_id);
        let mut step_outputs = Vec::new();
        let mut loop_ctx = RunLoop {
            run_id,
            workflow_id: workflow.id,
            state: &mut state,
            step_outputs: &mut step_outputs,
            run_input,
        };

        while let Some(current) = pending.pop() {
            if join_gate.is_join(&current) && !join_gate.is_ready(&current) {
                continue;
            }

            if let Err(err) = check_workflow_timeout(
                workflow_timeout,
                workflow_started,
                &run_key,
                &mut sprint5,
            ) {
                return Err(WorkflowRunError::Failed {
                    error: err,
                    partial: partial_record(&loop_ctx, &legacy),
                });
            }

            executor
                .record_visit(&current)
                .map_err(WorkflowRunError::Aborted)?;

            let Some(step) = node_to_step.get(current.as_str()) else {
                return Err(WorkflowRunError::Aborted(RuntimeError::InvalidWorkflowDefinition {
                    reason: format!("graph node '{current}' has no step mapping"),
                }));
            };
            let Some(agent) = agents.get(&step.agent_id) else {
                return Err(WorkflowRunError::Aborted(RuntimeError::AgentNotFound {
                    agent_id: step.agent_id,
                    step_id: step.id,
                }));
            };

            run_one_step(
                agent_runtime,
                &mut loop_ctx,
                step,
                graph_step_index,
                agent,
                &workflow.steps,
                agents,
                &mut memory,
                &mut legacy,
                &mut sprint5,
                &run_key,
                tool_runtime,
                tool_invoker.clone(),
                workflow_started,
                provider.clone(),
                provider_max_tokens,
                provider_temperature,
                retry_config.clone(),
                step_timeout,
                workflow_deadline,
                exec_config.recovery_enabled,
                None,
            )?;

            graph_step_index += 1;
            join_gate.mark_completed(&current);

            let checkpoint_outputs: Vec<_> = loop_ctx.step_outputs.to_vec();
            persist_graph_checkpoint(
                exec_config.recovery_enabled,
                workflow.id,
                run_id,
                run_input,
                &checkpoint_outputs,
                &current,
                executor.total_visits(),
                None,
            );

            let edge_key = loop_ctx
                .step_outputs
                .last()
                .map(|o| o.content.trim().to_string())
                .filter(|s| !s.is_empty());

            let next_nodes = executor
                .resolve_next(&current, edge_key.as_deref())
                .map_err(WorkflowRunError::Aborted)?;

            join_gate.enqueue_targets(&mut pending, next_nodes);
            join_gate.enqueue_ready_joins(&mut pending);
        }

        let duration_ms = workflow_started.elapsed().as_millis() as u64;
        sprint5.emit(TraceEventKind::WorkflowCompleted {
            run_id: run_key.clone(),
            duration_ms,
            total_tokens: TokenUsage::default(),
        });
        legacy.workflow_completed();
        let trace_events = legacy.events().to_vec();
        drop(sprint5);
        store.mark_complete(&run_key);
        maybe_export_trace(&run_key);
        Ok(WorkflowExecutionRecord {
            run_id,
            workflow_id: workflow.id,
            step_outputs,
            final_state: state.snapshot(),
            trace_events,
        })
    })
    .unwrap_or_else(|| {
        Err(WorkflowRunError::Aborted(RuntimeError::StateCommitFailed {
            step_id: Uuid::nil(),
            reason: "trace store lock unavailable".into(),
        }))
    })
}
