//! Per-run execution options from SDK (Sprint 7).

use uuid::Uuid;

use crate::retry::{RetryConfig, TimeoutConfig};

#[derive(Debug, Clone, Default)]
pub struct ExecutionConfig {
    pub retry: Option<RetryConfig>,
    pub timeouts: TimeoutConfig,
    pub recovery_enabled: bool,
    /// When set (e.g. by HTTP server), trace and recovery use this run id.
    pub run_id: Option<Uuid>,
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
