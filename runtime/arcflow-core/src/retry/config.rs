//! Retry configuration validated at workflow start.

use serde::{Deserialize, Serialize};

use crate::constants::{
    RETRY_DEFAULT_BASE_MS, RETRY_DEFAULT_MAX_MS, RETRY_DEFAULT_MULTIPLIER_X100,
    RETRY_MAX_ALLOWED_ATTEMPTS,
};
use crate::error::RuntimeError;
use crate::rcs::types::RetryPolicy;

/// Runtime retry configuration for a workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff: BackoffStrategy,
}

impl RetryConfig {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        if self.max_attempts == 0 {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: "retry max_attempts must be at least 1".into(),
            });
        }
        if self.max_attempts > RETRY_MAX_ALLOWED_ATTEMPTS {
            return Err(RuntimeError::InvalidWorkflowDefinition {
                reason: format!(
                    "retry max_attempts exceeds maximum {}",
                    RETRY_MAX_ALLOWED_ATTEMPTS
                ),
            });
        }
        self.backoff.validate()
    }

    pub fn from_rcs(policy: &RetryPolicy) -> Self {
        Self {
            max_attempts: policy.max_attempts,
            backoff: BackoffStrategy::Exponential {
                base_ms: policy.backoff_ms,
                multiplier_x100: RETRY_DEFAULT_MULTIPLIER_X100,
                max_ms: policy.max_backoff_ms,
                jitter_ms: 0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Exponential {
        base_ms: u64,
        multiplier_x100: u64,
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

impl BackoffStrategy {
    pub fn default_exponential() -> Self {
        Self::Exponential {
            base_ms: RETRY_DEFAULT_BASE_MS,
            multiplier_x100: RETRY_DEFAULT_MULTIPLIER_X100,
            max_ms: RETRY_DEFAULT_MAX_MS,
            jitter_ms: 0,
        }
    }

    pub fn compute_delay_ms(&self, attempt: u32, jitter_seed: u64) -> u64 {
        let base_delay = match self {
            Self::Exponential {
                base_ms,
                multiplier_x100,
                max_ms,
                ..
            } => {
                let mut delay = *base_ms;
                for _ in 0..attempt.saturating_sub(1) {
                    delay = delay.saturating_mul(*multiplier_x100).saturating_div(100);
                    if delay >= *max_ms {
                        delay = *max_ms;
                        break;
                    }
                }
                delay.min(*max_ms)
            }
            Self::Linear {
                base_ms,
                increment_ms,
                max_ms,
                ..
            } => {
                let increment = increment_ms.saturating_mul(attempt.saturating_sub(1) as u64);
                base_ms.saturating_add(increment).min(*max_ms)
            }
            Self::Constant { delay_ms, .. } => *delay_ms,
        };

        let jitter_ms = match self {
            Self::Exponential { jitter_ms, .. }
            | Self::Linear { jitter_ms, .. }
            | Self::Constant { jitter_ms, .. } => *jitter_ms,
        };
        if jitter_ms == 0 || jitter_seed == 0 {
            return base_delay;
        }
        base_delay.saturating_add(jitter_seed % jitter_ms)
    }

    fn validate(&self) -> Result<(), RuntimeError> {
        match self {
            Self::Exponential {
                base_ms,
                multiplier_x100,
                max_ms,
                ..
            } => {
                if *base_ms == 0 {
                    return Err(RuntimeError::InvalidWorkflowDefinition {
                        reason: "ExponentialBackoff base_ms must be at least 1".into(),
                    });
                }
                if *multiplier_x100 < 100 {
                    return Err(RuntimeError::InvalidWorkflowDefinition {
                        reason: "ExponentialBackoff multiplier must be at least 1.0".into(),
                    });
                }
                if *max_ms <= *base_ms {
                    return Err(RuntimeError::InvalidWorkflowDefinition {
                        reason: "ExponentialBackoff max_ms must exceed base_ms".into(),
                    });
                }
            }
            Self::Linear {
                base_ms, max_ms, ..
            } => {
                if *base_ms == 0 {
                    return Err(RuntimeError::InvalidWorkflowDefinition {
                        reason: "LinearBackoff base_ms must be at least 1".into(),
                    });
                }
                if *max_ms < *base_ms {
                    return Err(RuntimeError::InvalidWorkflowDefinition {
                        reason: "LinearBackoff max_ms must be >= base_ms".into(),
                    });
                }
            }
            Self::Constant { delay_ms, .. } => {
                if *delay_ms == 0 {
                    return Err(RuntimeError::InvalidWorkflowDefinition {
                        reason: "ConstantBackoff delay_ms must be at least 1".into(),
                    });
                }
            }
        }
        Ok(())
    }
}
