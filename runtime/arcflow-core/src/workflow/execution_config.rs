//! Per-run execution options from SDK (Sprint 7).

use crate::retry::{RetryConfig, TimeoutConfig};

#[derive(Debug, Clone, Default)]
pub struct ExecutionConfig {
    pub retry: Option<RetryConfig>,
    pub timeouts: TimeoutConfig,
    pub recovery_enabled: bool,
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
