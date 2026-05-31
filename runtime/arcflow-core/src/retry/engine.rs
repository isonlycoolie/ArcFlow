//! Retry loop with trace emission before each backoff sleep.

use std::future::Future;

use crate::providers::error::ProviderCallError;
use crate::retry::config::RetryConfig;
use crate::tracing::events::TraceEventKind;
use crate::tracing::sprint5_emitter::TraceEventEmitter;

pub trait Retryable {
    fn is_retryable(&self) -> bool;
    fn error_code(&self) -> String;
}

impl Retryable for ProviderCallError {
    fn is_retryable(&self) -> bool {
        self.is_retryable()
    }

    fn error_code(&self) -> String {
        match self {
            ProviderCallError::RateLimited { .. } => "provider_rate_limited".into(),
            ProviderCallError::Timeout { .. } => "provider_timeout".into(),
            ProviderCallError::NetworkError { .. } => "provider_network_error".into(),
            ProviderCallError::AuthenticationFailed { .. } => "provider_auth_failed".into(),
            ProviderCallError::ApiError { status_code, .. } => {
                format!("provider_api_error_{status_code}")
            }
            ProviderCallError::ResponseParseError { .. } => "provider_response_parse_error".into(),
            ProviderCallError::NotConfigured { .. } => "provider_not_configured".into(),
            ProviderCallError::ContentFiltered { .. } => "provider_content_filtered".into(),
        }
    }
}

#[derive(Debug)]
pub enum RetryError<E> {
    NonRetryable(E),
    Exhausted {
        attempts_made: u32,
        last_error_code: String,
        error: E,
    },
}

impl<E: Retryable> RetryError<E> {
    pub fn is_exhausted(&self) -> bool {
        matches!(self, RetryError::Exhausted { .. })
    }

    pub fn attempts_made(&self) -> u32 {
        match self {
            RetryError::NonRetryable(_) => 1,
            RetryError::Exhausted { attempts_made, .. } => *attempts_made,
        }
    }
}

pub async fn execute_with_retry<F, Fut, T, E>(
    operation: F,
    config: &RetryConfig,
    step_id: &str,
    emitter: &mut TraceEventEmitter<'_>,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Retryable + std::fmt::Debug,
{
    let run_id = emitter.trace_id().to_string();

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if !err.is_retryable() {
                    return Err(RetryError::NonRetryable(err));
                }
                if attempt == config.max_attempts {
                    let error_code = err.error_code();
                    emitter.emit(TraceEventKind::RetryExhausted {
                        run_id: run_id.clone(),
                        step_id: step_id.to_string(),
                        total_attempts: attempt,
                        last_error_code: error_code.clone(),
                    });
                    return Err(RetryError::Exhausted {
                        attempts_made: attempt,
                        last_error_code: error_code,
                        error: err,
                    });
                }
                let jitter_seed = jitter_seed_from_time();
                let backoff_ms = config.backoff.compute_delay_ms(attempt, jitter_seed);
                let error_code = err.error_code();
                emitter.emit(TraceEventKind::RetryAttempted {
                    run_id: run_id.clone(),
                    step_id: step_id.to_string(),
                    attempt_number: attempt,
                    max_attempts: config.max_attempts,
                    backoff_ms,
                    trigger_error_code: error_code,
                });
                #[cfg(feature = "otel")]
                crate::tracing::otel_metrics::record_retry_attempt(step_id);
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
            }
        }
    }
    unreachable!("retry loop must return before exhaustion of attempts")
}

fn jitter_seed_from_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(42)
}
