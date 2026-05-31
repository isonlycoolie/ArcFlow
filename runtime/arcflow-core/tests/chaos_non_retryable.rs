//! C3 — non-retryable errors fail immediately.

use std::time::Instant;

use arcflow_core::providers::error::ProviderCallError;
use arcflow_core::retry::config::{BackoffStrategy, RetryConfig};
use arcflow_core::retry::{execute_with_retry, RetryError};
use arcflow_core::tracing::sprint5_emitter::TraceEventEmitter;
use arcflow_core::tracing::store::TraceStore;

#[tokio::test]
async fn chaos_non_retryable_fast_fail() {
    let started = Instant::now();
    let mut store = TraceStore::new();
    let mut emitter = TraceEventEmitter::new("chaos-c3".into(), &mut store);
    let config = RetryConfig {
        max_attempts: 5,
        backoff: BackoffStrategy::Constant {
            delay_ms: 500,
            jitter_ms: 0,
        },
    };
    let err = execute_with_retry(
        || async {
            Err(ProviderCallError::ApiError {
                provider_id: "mock".into(),
                status_code: 400,
                sanitized_message: "bad request".into(),
            })
        },
        &config,
        "step_1",
        &mut emitter,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, RetryError::NonRetryable(_)));
    assert!(started.elapsed().as_millis() < 200);
}
