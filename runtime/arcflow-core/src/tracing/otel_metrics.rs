//! OpenTelemetry metrics (opt-in via `ARCFLOW_OTEL_ENABLED`).
#![cfg(feature = "otel")]

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;

use super::otel_config::{self, OtlpProtocol};

static METER: OnceLock<Meter> = OnceLock::new();
static WORKFLOW_DURATION: OnceLock<Histogram<u64>> = OnceLock::new();
static LLM_TOKENS: OnceLock<Counter<u64>> = OnceLock::new();

fn metrics_resource() -> Resource {
    let mut attrs = vec![KeyValue::new(
        "service.name",
        otel_config::service_name(),
    )];
    for (key, value) in otel_config::resource_attributes() {
        attrs.push(KeyValue::new(key, value));
    }
    Resource::new(attrs)
}

fn build_metric_exporter(endpoint: &str) -> Result<opentelemetry_otlp::MetricExporter, String> {
    let timeout = Duration::from_millis(crate::constants::OTLP_EXPORT_TIMEOUT_MS);
    match otel_config::otlp_protocol() {
        OtlpProtocol::Grpc => opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_timeout(timeout)
            .build()
            .map_err(|e| format!("otlp grpc metric exporter: {e}")),
        OtlpProtocol::HttpProtobuf => opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .with_timeout(timeout)
            .build()
            .map_err(|e| format!("otlp http metric exporter: {e}")),
    }
}

pub fn init_metrics() -> Result<(), String> {
    if METER.get().is_some() {
        return Ok(());
    }
    let Some(endpoint) = otel_config::otlp_endpoint() else {
        return Err("OTEL_EXPORTER_OTLP_ENDPOINT not set".into());
    };
    let exporter = build_metric_exporter(&endpoint)?;
    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();
    let provider = SdkMeterProvider::builder()
        .with_resource(metrics_resource())
        .with_reader(reader)
        .build();
    global::set_meter_provider(provider);
    let meter = global::meter("arcflow-core");
    WORKFLOW_DURATION
        .set(meter.u64_histogram("arcflow.workflow.duration_ms").build())
        .ok();
    LLM_TOKENS
        .set(
            meter
                .u64_counter("arcflow.llm.tokens")
                .with_description("LLM token usage by direction")
                .build(),
        )
        .ok();
    METER.set(meter).map_err(|_| "metrics meter already set".to_string())
}

pub fn record_workflow_duration_ms(duration_ms: u64, status: &str, workflow_name: &str) {
    if !otel_config::otel_enabled() {
        return;
    }
    let _ = init_metrics();
    if let Some(hist) = WORKFLOW_DURATION.get() {
        hist.record(
            duration_ms,
            &[
                KeyValue::new("status", status.to_string()),
                KeyValue::new("workflow_name", workflow_name.to_string()),
            ],
        );
    }
}

pub fn record_llm_tokens(provider: &str, model: &str, direction: &str, count: u64) {
    if !otel_config::otel_enabled() || count == 0 {
        return;
    }
    let _ = init_metrics();
    if let Some(counter) = LLM_TOKENS.get() {
        counter.add(
            count,
            &[
                KeyValue::new("provider", provider.to_string()),
                KeyValue::new("model", model.to_string()),
                KeyValue::new("direction", direction.to_string()),
            ],
        );
    }
}
