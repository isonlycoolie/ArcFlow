//! OpenTelemetry integration tests (feature `otel`).
#![cfg(feature = "otel")]

use std::sync::Mutex;

use arcflow_core::tracing::otel_config;
use arcflow_core::tracing::otel_inmemory::{finished_spans, install_inmemory_provider};
use arcflow_core::tracing::otel_live::{step_span, workflow_span};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_clean_env(test: impl FnOnce()) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    for key in [
        "ARCFLOW_OTEL_ENABLED",
        "ARCFLOW_OTLP_ENDPOINT",
        "OTEL_EXPORTER_OTLP_ENDPOINT",
    ] {
        std::env::remove_var(key);
    }
    test();
    for key in [
        "ARCFLOW_OTEL_ENABLED",
        "ARCFLOW_OTLP_ENDPOINT",
        "OTEL_EXPORTER_OTLP_ENDPOINT",
    ] {
        std::env::remove_var(key);
    }
}

#[test]
fn otel_spans_noop_when_disabled() {
    with_clean_env(|| {
        let (exporter, _provider) = install_inmemory_provider();
        {
            let _wf = workflow_span("run-1", "demo", None);
            let _step = step_span("run-1", "step-a", 0, "agent");
        }
        assert!(finished_spans(&exporter).is_empty());
    });
}

#[test]
fn otel_export_not_requested_by_default() {
    with_clean_env(|| {
        assert!(!otel_config::export_requested());
    });
}

#[test]
fn otel_enabled_requires_explicit_switch() {
    with_clean_env(|| {
        assert!(!otel_config::otel_enabled());
        std::env::set_var("ARCFLOW_OTEL_ENABLED", "true");
        assert!(otel_config::otel_enabled());
    });
}
