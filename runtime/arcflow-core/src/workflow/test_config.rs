//! Deterministic workflow test stubs (Phase 2.3).

use std::collections::HashMap;

use serde::Deserialize;

/// Per-step stub behaviour for `workflow.test()`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TestStubStep {
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub fail_times: Option<u32>,
    #[serde(default)]
    pub then_output: Option<String>,
}

/// Test execution overrides (no production Postgres writes when enabled).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TestConfig {
    #[serde(default)]
    pub stub_responses: HashMap<String, TestStubStep>,
}

impl TestConfig {
    pub fn stub_output(&self, key: &str, attempt: u32) -> Option<String> {
        let step = self.stub_responses.get(key)?;
        if let Some(fail_times) = step.fail_times {
            if attempt <= fail_times {
                return None;
            }
            return step
                .then_output
                .clone()
                .or_else(|| step.output.clone());
        }
        step.output.clone()
    }

    pub fn should_fail(&self, key: &str, attempt: u32) -> bool {
        self.stub_responses
            .get(key)
            .and_then(|s| s.fail_times)
            .is_some_and(|n| attempt <= n)
    }
}

/// Keys: `step_{order}` or step UUID string.
pub fn resolve_key(order: u32, step_id: &str, config: &TestConfig) -> Option<String> {
    let by_order = format!("step_{order}");
    if config.stub_responses.contains_key(&by_order) {
        return Some(by_order);
    }
    if config.stub_responses.contains_key(step_id) {
        return Some(step_id.to_string());
    }
    None
}
