//! Per-run execution options from SDK (Sprint 7).

use uuid::Uuid;

use crate::retry::{RetryConfig, TimeoutConfig};
use crate::workflow::test_config::TestConfig;

use std::sync::Arc;

use crate::debug::DebugSession;

/// SDK opt-in for user-facing stream events (Phase 2.1).
#[derive(Debug, Clone, Default)]
pub struct StreamConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionConfig {
    pub retry: Option<RetryConfig>,
    pub timeouts: TimeoutConfig,
    pub recovery_enabled: bool,
    /// When set (e.g. by HTTP server), trace and recovery use this run id.
    pub run_id: Option<Uuid>,
    /// Registry semver pin when the run used `workflow_ref` (Phase 3.3).
    pub workflow_version: Option<String>,
    /// Deterministic stub overrides for workflow.test() (Phase 2.3).
    pub test: Option<TestConfig>,
    /// When enabled, step and token events are emitted on the run stream channel.
    pub stream: Option<StreamConfig>,
    /// Local debug session with breakpoints (Phase 2.4).
    pub debug: Option<Arc<DebugSession>>,
    /// Initial graph run state JSON (Phase 2-Pro).
    pub initial_state: Option<serde_json::Map<String, serde_json::Value>>,
}

impl ExecutionConfig {
    pub fn validate(&self) -> Result<(), crate::error::RuntimeError> {
        if let Some(r) = &self.retry {
            r.validate()?;
        }
        self.timeouts.validate()?;
        Ok(())
    }
}
