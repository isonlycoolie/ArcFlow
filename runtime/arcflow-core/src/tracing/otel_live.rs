//! Live tracing → OpenTelemetry bridge (`tracing-opentelemetry`).
#![cfg(feature = "otel")]

use std::sync::Once;

use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

use super::otel::init_otlp_exporter;
use super::otel_config;

static INIT: Once = Once::new();

/// Span guard; no-op when OTel is disabled.
pub struct SpanGuard {
    _inner: Option<tracing::span::EnteredSpan>,
}

impl SpanGuard {
    fn none() -> Self {
        Self { _inner: None }
    }
}

fn try_init_live_tracing() {
    if !otel_config::otel_enabled() {
        return;
    }
    INIT.call_once(|| {
        if init_otlp_exporter().is_err() {
            return;
        }
        let tracer = super::otel::sdk_tracer("arcflow-core");
        let layer = OpenTelemetryLayer::new(tracer);
        let subscriber = Registry::default().with(layer);
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

/// Opens an `arcflow.workflow` span when OTel is enabled.
pub fn workflow_span(run_id: &str, workflow_name: &str) -> SpanGuard {
    if !otel_config::otel_enabled() {
        return SpanGuard::none();
    }
    try_init_live_tracing();
    SpanGuard {
        _inner: Some(
            tracing::info_span!(
                "arcflow.workflow",
                run_id = run_id,
                workflow_name = workflow_name,
            )
            .entered(),
        ),
    }
}

/// Opens an `arcflow.step` span nested under the active workflow span.
pub fn step_span(run_id: &str, step_id: &str, step_index: usize, agent_name: &str) -> SpanGuard {
    if !otel_config::otel_enabled() {
        return SpanGuard::none();
    }
    try_init_live_tracing();
    SpanGuard {
        _inner: Some(
            tracing::info_span!(
                "arcflow.step",
                run_id = run_id,
                step_id = step_id,
                step_index = step_index,
                agent_name = agent_name,
            )
            .entered(),
        ),
    }
}

/// Opens an `arcflow.llm.invoke` span under the active step span.
pub fn llm_span(run_id: &str, step_id: &str, provider: &str, model: &str) -> SpanGuard {
    if !otel_config::otel_enabled() {
        return SpanGuard::none();
    }
    try_init_live_tracing();
    SpanGuard {
        _inner: Some(
            tracing::info_span!(
                "arcflow.llm.invoke",
                run_id = run_id,
                step_id = step_id,
                provider = provider,
                model = model,
                tokens.prompt = tracing::field::Empty,
                tokens.completion = tracing::field::Empty,
            )
            .entered(),
        ),
    }
}

/// Records token counts on the active LLM span (SEC-1: counts only).
pub fn record_llm_tokens(prompt_tokens: u32, completion_tokens: u32) {
    if !otel_config::otel_enabled() {
        return;
    }
    let span = tracing::Span::current();
    span.record("tokens.prompt", prompt_tokens);
    span.record("tokens.completion", completion_tokens);
}
