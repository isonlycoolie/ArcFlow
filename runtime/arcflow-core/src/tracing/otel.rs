//! OpenTelemetry OTLP export (ADR-009). Requires `otel` Cargo feature.
#![cfg(feature = "otel")]

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::trace::{SimpleSpanProcessor, TracerProvider};
use opentelemetry_sdk::Resource;

use crate::constants::OTLP_EXPORT_TIMEOUT_MS;

use super::otel_config::{self, OtlpProtocol};

static PROVIDER: OnceLock<TracerProvider> = OnceLock::new();

/// Returns true when an OTLP endpoint is configured and export was requested.
pub fn otlp_configured() -> bool {
    otel_config::export_requested() && otel_config::otlp_endpoint().is_some()
}

fn build_resource() -> Resource {
    let mut attrs = vec![KeyValue::new("service.name", otel_config::service_name())];
    for (key, value) in otel_config::resource_attributes() {
        attrs.push(KeyValue::new(key, value));
    }
    Resource::new(attrs)
}

fn build_exporter(endpoint: &str) -> Result<opentelemetry_otlp::SpanExporter, String> {
    let timeout = Duration::from_millis(OTLP_EXPORT_TIMEOUT_MS);
    match otel_config::otlp_protocol() {
        OtlpProtocol::Grpc => opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_timeout(timeout)
            .build()
            .map_err(|e| format!("otlp grpc exporter build: {e}")),
        OtlpProtocol::HttpProtobuf => opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(endpoint)
            .with_timeout(timeout)
            .build()
            .map_err(|e| format!("otlp http exporter build: {e}")),
    }
}

/// Initializes the global OTLP tracer provider (once per process).
pub fn init_otlp_exporter() -> Result<(), String> {
    if PROVIDER.get().is_some() {
        return Ok(());
    }
    let Some(endpoint) = otel_config::otlp_endpoint() else {
        return Err("OTEL_EXPORTER_OTLP_ENDPOINT not set".into());
    };
    let exporter = build_exporter(&endpoint)?;
    let provider = TracerProvider::builder()
        .with_resource(build_resource())
        .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter)))
        .build();
    global::set_tracer_provider(provider.clone());
    PROVIDER
        .set(provider)
        .map_err(|_| "otlp provider already initialized".to_string())
}

/// SDK tracer for live `tracing-opentelemetry` instrumentation.
pub fn sdk_tracer(instrumentation: &'static str) -> opentelemetry_sdk::trace::Tracer {
    let _ = init_otlp_exporter();
    if let Some(provider) = PROVIDER.get() {
        return provider.tracer(instrumentation);
    }
    TracerProvider::builder().build().tracer(instrumentation)
}
