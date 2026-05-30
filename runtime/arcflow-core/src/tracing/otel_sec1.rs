//! SEC-1 span attribute audit tests.
#![cfg(feature = "otel")]

const FORBIDDEN_ATTR_FRAGMENTS: &[&str] = &[
    "prompt.text",
    "completion.text",
    "prompt_text",
    "completion_text",
];

/// Returns true when serialized attributes contain no SEC-1 violations.
pub fn attributes_are_sec1_clean(attrs_debug: &str) -> bool {
    let lower = attrs_debug.to_lowercase();
    !FORBIDDEN_ATTR_FRAGMENTS
        .iter()
        .any(|term| lower.contains(term))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::otel_export_impl::step_metadata_attrs;
    use crate::tracing::types::{StepExecutionStatus, StepTrace, TokenUsage};

    fn sample_step(instructions: &str) -> StepTrace {
        StepTrace {
            step_index: 0,
            step_id: "step-0".into(),
            agent_name: "agent".into(),
            agent_role: instructions.into(),
            status: StepExecutionStatus::Completed,
            started_at: chrono::Utc::now(),
            completed_at: None,
            duration_ms: Some(42),
            tokens: TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            },
            tool_calls: Vec::new(),
            memory_operations: Vec::new(),
            error: None,
        }
    }

    #[test]
    fn step_metadata_attrs_pass_sec1_audit() {
        let secret = "super-secret-instructions-do-not-export";
        let attrs = step_metadata_attrs(&sample_step("researcher"));
        let blob: String = attrs.iter().map(|kv| format!("{:?}", kv)).collect();
        assert!(attributes_are_sec1_clean(&blob));
        assert!(!blob.contains(secret));
        assert!(blob.contains("duration_ms"));
    }

    #[test]
    fn token_counts_allowed_without_prompt_body() {
        let attrs = step_metadata_attrs(&sample_step("researcher"));
        let blob: String = attrs.iter().map(|kv| format!("{:?}", kv)).collect();
        assert!(!blob.contains("prompt.text"));
        assert!(!blob.contains("completion.text"));
    }
}
