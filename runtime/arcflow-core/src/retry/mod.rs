//! Retry engine — exclusive location for retry logic (Sprint 7, ADR-013).

pub mod config;
pub mod engine;
pub mod timeout;

pub use config::{BackoffStrategy, RetryConfig};
pub use engine::{execute_with_retry, RetryError, Retryable};
pub use timeout::{with_step_timeout, TimeoutConfig};
