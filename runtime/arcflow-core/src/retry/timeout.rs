//! Step and workflow timeout enforcement.

use std::future::Future;
use std::time::{Duration, Instant};

use crate::constants::TIMEOUT_MAX_SECONDS;
use crate::error::RuntimeError;
use crate::tracing::events::TraceEventKind;
use crate::tracing::sprint5_emitter::TraceEventEmitter;

#[derive(Debug, Clone, Default)]
pub struct TimeoutConfig {
    pub workflow_timeout: Option<Duration>,
    pub step_timeout: Option<Duration>,
}

impl TimeoutConfig {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        if let Some(wt) = self.workflow_timeout {
            validate_duration(wt, "workflow")?;
        }
        if let Some(st) = self.step_timeout {
            validate_duration(st, "step")?;
        }
        Ok(())
    }
}

fn validate_duration(d: Duration, label: &str) -> Result<(), RuntimeError> {
    if d.as_secs_f64() <= 0.0 {
        return Err(RuntimeError::InvalidWorkflowDefinition {
            reason: format!("{label} timeout must be positive"),
        });
    }
    if d.as_secs() > TIMEOUT_MAX_SECONDS {
        return Err(RuntimeError::InvalidWorkflowDefinition {
            reason: format!(
                "{label} timeout {}s exceeds maximum {}s",
                d.as_secs(),
                TIMEOUT_MAX_SECONDS
            ),
        });
    }
    Ok(())
}

pub async fn with_step_timeout<F, Fut, T>(
    operation: F,
    timeout: Duration,
    step_id: &str,
    emitter: &mut TraceEventEmitter<'_>,
) -> Result<T, RuntimeError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, RuntimeError>>,
{
    let start = Instant::now();
    let run_id = emitter.trace_id().to_string();
    match tokio::time::timeout(timeout, operation()).await {
        Ok(result) => result,
        Err(_elapsed) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            let configured_ms = timeout.as_millis() as u64;
            emitter.emit(TraceEventKind::TimeoutEnforced {
                run_id: run_id.clone(),
                step_id: step_id.to_string(),
                timeout_type: "step".to_string(),
                configured_ms,
                elapsed_ms,
            });
            Err(RuntimeError::StepTimeout {
                step_id: step_id.to_string(),
                configured_ms,
                elapsed_ms,
            })
        }
    }
}
