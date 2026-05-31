//! C2 — retries exhaust on persistent rate limit.

use arcflow_core::providers::error::ProviderCallError;
use arcflow_core::retry::config::{BackoffStrategy, RetryConfig};
use arcflow_core::retry::{execute_with_retry, RetryError};
use arcflow_core::tracing::sprint5_emitter::TraceEventEmitter;
use arcflow_core::tracing::store::TraceStore;

#[tokio::test]
async fn chaos_retry_exhausted() {
    let mut store = TraceStore::new();
    let mut emitter = TraceEventEmitter::new("chaos-c2".into(), &mut store);
    let config = RetryConfig {
        max_attempts: 3,
        backoff: BackoffStrategy::Constant {
            delay_ms: 0,
            jitter_ms: 0,
        },
    };
    let err = execute_with_retry(
        || async {
            Err(ProviderCallError::RateLimited {
                provider_id: "mock".into(),
                retry_after_seconds: None,
            })
        },
        &config,
        "step_1",
        &mut emitter,
    )
    .await
    .unwrap_err();
    match err {
        RetryError::Exhausted { attempts_made, .. } => assert_eq!(attempts_made, 3),
        other => panic!("expected exhausted, got {other:?}"),
    }
}
