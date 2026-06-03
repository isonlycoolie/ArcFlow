//! In-memory span exporter helpers for OTel unit tests.
#![cfg(feature = "otel")]

use opentelemetry::global;
use opentelemetry::trace::Tracer;
use opentelemetry_sdk::export::trace::SpanData;
use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
use opentelemetry_sdk::trace::{SimpleSpanProcessor, TracerProvider};

/// Installs a process-global in-memory tracer provider.
pub fn install_inmemory_provider() -> (InMemorySpanExporter, TracerProvider) {
    let exporter = InMemorySpanExporter::default();
    let provider = TracerProvider::builder()
        .with_span_processor(SimpleSpanProcessor::new(Box::new(exporter.clone())))
        .build();
    let _ = global::set_tracer_provider(provider.clone());
    (exporter, provider)
}

/// Returns finished spans captured by the in-memory exporter.
pub fn finished_spans(exporter: &InMemorySpanExporter) -> Vec<SpanData> {
    exporter.get_finished_spans().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inmemory_exporter_captures_span() {
        let (exporter, _provider) = install_inmemory_provider();
        let tracer = global::tracer("arcflow-test");
        tracer.in_span("arcflow.workflow", |_| {});
        let spans = finished_spans(&exporter);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "arcflow.workflow");
    }

    #[test]
    fn workflow_step_llm_span_hierarchy() {
        let (exporter, _provider) = install_inmemory_provider();
        let tracer = global::tracer("arcflow-test");
        tracer.in_span("arcflow.workflow", |_| {
            tracer.in_span("arcflow.step", |_| {
                tracer.in_span("arcflow.llm.invoke", |_| {});
            });
        });
        let spans = finished_spans(&exporter);
        assert_eq!(spans.len(), 3);
        let names: Vec<String> = spans.iter().map(|s| s.name.to_string()).collect();
        assert!(names.iter().any(|n| n == "arcflow.workflow"));
        assert!(names.iter().any(|n| n == "arcflow.step"));
        assert!(names.iter().any(|n| n == "arcflow.llm.invoke"));
    }
}
