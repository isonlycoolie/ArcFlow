//! C1 — rate-limited provider succeeds on retry.

use std::sync::atomic::{AtomicU32, Ordering};

use arcflow_core::providers::error::ProviderCallError;
use arcflow_core::retry::config::{BackoffStrategy, RetryConfig};
use arcflow_core::retry::execute_with_retry;
use arcflow_core::tracing::sprint5_emitter::TraceEventEmitter;
use arcflow_core::tracing::store::TraceStore;

static CALLS: AtomicU32 = AtomicU32::new(0);

#[tokio::test]
async fn chaos_rate_limit_then_success() {
    CALLS.store(0, Ordering::SeqCst);
    let mut store = TraceStore::new();
    let mut emitter = TraceEventEmitter::new("chaos-c1".into(), &mut store);
    let config = RetryConfig {
        max_attempts: 3,
        backoff: BackoffStrategy::Constant {
            delay_ms: 0,
            jitter_ms: 0,
        },
    };
    let result = execute_with_retry(
        || async {
            let n = CALLS.fetch_add(1, Ordering::SeqCst) + 1;
            if n == 1 {
                Err(ProviderCallError::RateLimited {
                    provider_id: "mock".into(),
                    retry_after_seconds: None,
                })
            } else {
                Ok("ok")
            }
        },
        &config,
        "step_1",
        &mut emitter,
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(CALLS.load(Ordering::SeqCst), 2);
}
