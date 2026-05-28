//! OpenTelemetry OTLP export (ADR-009). Active only when `ARCFLOW_OTLP_ENDPOINT` is set.

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{SimpleSpanProcessor, TracerProvider};

use crate::constants::OTLP_EXPORT_TIMEOUT_MS;

static PROVIDER: OnceLock<TracerProvider> = OnceLock::new();

/// Returns true when `ARCFLOW_OTLP_ENDPOINT` is set.
pub fn otlp_configured() -> bool {
    std::env::var("ARCFLOW_OTLP_ENDPOINT")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

fn endpoint() -> Option<String> {
    std::env::var("ARCFLOW_OTLP_ENDPOINT")
        .ok()
        .filter(|v| !v.trim().is_empty())
}

/// Initializes the global OTLP tracer provider (once per process).
pub fn init_otlp_exporter() -> Result<(), String> {
    if PROVIDER.get().is_some() {
        return Ok(());
    }
    let Some(endpoint) = endpoint() else {
        return Err("ARCFLOW_OTLP_ENDPOINT not set".into());
    };
    let timeout = Duration::from_millis(OTLP_EXPORT_TIMEOUT_MS);
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(timeout)
        .build()
        .map_err(|e| format!("otlp exporter build: {e}"))?;
    let provider = TracerProvider::builder()
        .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter)))
        .build();
    global::set_tracer_provider(provider.clone());
    PROVIDER
        .set(provider)
        .map_err(|_| "otlp provider already initialized".to_string())
}
