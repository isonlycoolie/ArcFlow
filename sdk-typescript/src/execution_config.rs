//! Parse Sprint 7 execution options from JSON (SDK → runtime).

use std::time::Duration;

use arcflow_core::retry::{BackoffStrategy, RetryConfig};
use arcflow_core::retry::TimeoutConfig as RetryTimeoutConfig;
use arcflow_core::workflow::ExecutionConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ExecConfigJson {
    retry: Option<RetryJson>,
    workflow_timeout_secs: Option<f64>,
    step_timeout_secs: Option<f64>,
    recovery_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct RetryJson {
    max_attempts: u32,
    backoff: BackoffJson,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BackoffJson {
    Exponential {
        base_ms: u64,
        multiplier: f64,
        max_ms: u64,
        jitter_ms: u64,
    },
    Linear {
        base_ms: u64,
        increment_ms: u64,
        max_ms: u64,
        jitter_ms: u64,
    },
    Constant {
        delay_ms: u64,
        jitter_ms: u64,
    },
}

impl BackoffJson {
    fn into_strategy(self) -> BackoffStrategy {
        match self {
            BackoffJson::Exponential {
                base_ms,
                multiplier,
                max_ms,
                jitter_ms,
            } => BackoffStrategy::Exponential {
                base_ms,
                multiplier_x100: (multiplier * 100.0) as u64,
                max_ms,
                jitter_ms,
            },
            BackoffJson::Linear {
                base_ms,
                increment_ms,
                max_ms,
                jitter_ms,
            } => BackoffStrategy::Linear {
                base_ms,
                increment_ms,
                max_ms,
                jitter_ms,
            },
            BackoffJson::Constant {
                delay_ms,
                jitter_ms,
            } => BackoffStrategy::Constant { delay_ms, jitter_ms },
        }
    }
}

pub fn parse_execution_config(raw: Option<&str>) -> Result<ExecutionConfig, String> {
    let Some(json) = raw.filter(|s| !s.trim().is_empty()) else {
        return Ok(ExecutionConfig::default());
    };
    let parsed: ExecConfigJson =
        serde_json::from_str(json).map_err(|e| format!("Invalid execution config JSON: {e}"))?;
    let retry = parsed.retry.map(|r| RetryConfig {
        max_attempts: r.max_attempts,
        backoff: r.backoff.into_strategy(),
    });
    let mut timeouts = RetryTimeoutConfig::default();
    if let Some(secs) = parsed.workflow_timeout_secs {
        timeouts.workflow_timeout = Some(Duration::from_secs_f64(secs));
    }
    if let Some(secs) = parsed.step_timeout_secs {
        timeouts.step_timeout = Some(Duration::from_secs_f64(secs));
    }
    Ok(ExecutionConfig {
        retry,
        timeouts,
        recovery_enabled: parsed.recovery_enabled.unwrap_or(false),
    })
}
