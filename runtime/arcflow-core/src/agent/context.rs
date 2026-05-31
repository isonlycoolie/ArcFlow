//! Prompt-safe context assembly (Phase 2-Pro).

use crate::rcs::types::{ContextPolicy, PriorStepsMode};
use crate::state::StateSnapshot;

/// Optional context blocks merged into the agent prompt.
#[derive(Debug, Clone, Default)]
pub struct ContextExtras {
    pub memory_note: Option<String>,
    pub rag_block: Option<String>,
    pub tool_results: Option<String>,
    pub graph_state: Option<String>,
}

/// Builds ordered, prompt-safe context from run input, state, and extras.
pub struct ContextAssembler;

impl ContextAssembler {
    pub fn effective_policy(agent_policy: Option<&ContextPolicy>) -> ContextPolicy {
        agent_policy.cloned().unwrap_or_default()
    }

    pub fn assemble(
        run_input: &str,
        state: &StateSnapshot,
        policy: &ContextPolicy,
        extras: &ContextExtras,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        if policy.include_run_input && !run_input.is_empty() {
            parts.push(format!("## Run input\n{run_input}"));
        }

        let prior = Self::prior_steps_block(state, policy);
        if !prior.is_empty() {
            parts.push(format!("## Prior steps\n{prior}"));
        }

        if let Some(gs) = extras.graph_state.as_ref().filter(|s| !s.is_empty()) {
            parts.push(format!("## Graph state\n{gs}"));
        }

        if let Some(mem) = extras.memory_note.as_ref().filter(|s| !s.is_empty()) {
            parts.push(format!("## Memory\n{mem}"));
        }

        if let Some(rag) = extras.rag_block.as_ref().filter(|s| !s.is_empty()) {
            parts.push(format!("## Retrieved knowledge\n{rag}"));
        }

        if let Some(tools) = extras.tool_results.as_ref().filter(|s| !s.is_empty()) {
            parts.push(format!("## Tool results\n{tools}"));
        }

        if parts.is_empty() {
            run_input.to_string()
        } else {
            parts.join("\n\n")
        }
    }

    fn prior_steps_block(state: &StateSnapshot, policy: &ContextPolicy) -> String {
        if matches!(policy.include_prior_steps, PriorStepsMode::None) {
            return String::new();
        }
        let steps: Vec<_> = match policy.include_prior_steps {
            PriorStepsMode::All => state.steps.iter().collect(),
            PriorStepsMode::Last => state.steps.last().into_iter().collect(),
            PriorStepsMode::None => vec![],
        };
        let mut out = String::new();
        let mut budget = policy.max_prior_step_chars;
        for (i, step) in steps.iter().enumerate() {
            let line = format!("Step {}: {}\n", i + 1, step.content);
            if line.len() > budget {
                if budget > 0 {
                    let take = budget.min(line.len());
                    out.push_str(&line[..take]);
                    out.push('…');
                }
                break;
            }
            out.push_str(&line);
            budget = budget.saturating_sub(line.len());
        }
        out.trim().to_string()
    }

    /// User query for vector retrieval (fallback: first line of assembled context).
    pub fn query_for_retrieval(run_input: &str, assembled: &str) -> String {
        if !run_input.is_empty() {
            run_input.to_string()
        } else {
            assembled.lines().next().unwrap_or("").to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rcs::types::ExecutionStatus;
    use crate::state::ExecutionStepOutput;
    use uuid::Uuid;

    #[test]
    fn prior_steps_included_in_assembled_context() {
        let step_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let state = StateSnapshot {
            steps: vec![ExecutionStepOutput {
                step_id,
                agent_id,
                content: "first output".into(),
                status: ExecutionStatus::Completed,
            }],
        };
        let policy = ContextPolicy::default();
        let ctx = ContextAssembler::assemble("query", &state, &policy, &ContextExtras::default());
        assert!(ctx.contains("first output"));
        assert!(ctx.contains("## Prior steps"));
    }

    #[test]
    fn assembled_context_includes_rag_block() {
        let state = StateSnapshot { steps: vec![] };
        let policy = ContextPolicy::default();
        let extras = ContextExtras {
            rag_block: Some("Module 3 covers RAG.".into()),
            ..ContextExtras::default()
        };
        let ctx = ContextAssembler::assemble("query", &state, &policy, &extras);
        assert!(ctx.contains("## Retrieved knowledge"));
        assert!(ctx.contains("Module 3 covers RAG."));
    }
}
