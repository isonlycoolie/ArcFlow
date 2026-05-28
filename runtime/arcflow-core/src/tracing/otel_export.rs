//! Maps ExecutionTrace to OTLP spans (metadata only).

use opentelemetry::trace::{TraceContextExt, Tracer, TracerProvider as _};
use opentelemetry::KeyValue;
use tracing::warn;

use super::otel::{init_otlp_exporter, otlp_configured};
use super::registry::get_execution_trace;
use super::types::{ExecutionTrace, StepTrace};

/// Metadata attribute keys allowed on exported spans (SEC-1).
pub fn step_metadata_attrs(step: &StepTrace) -> Vec<KeyValue> {
    vec![
        KeyValue::new("step_index", step.step_index as i64),
        KeyValue::new("step_id", step.step_id.clone()),
        KeyValue::new("agent_name", step.agent_name.clone()),
        KeyValue::new("agent_role", step.agent_role.clone()),
        KeyValue::new("status", format!("{:?}", step.status)),
        KeyValue::new("duration_ms", step.duration_ms.unwrap_or(0) as i64),
    ]
}

fn export_trace_blocking(trace: &ExecutionTrace) -> Result<(), String> {
    init_otlp_exporter()?;
    let provider = opentelemetry::global::tracer_provider();
    let tracer = provider.tracer("arcflow-core");
    tracer.in_span(format!("workflow/{}", trace.workflow_name), |cx| {
        let span = cx.span();
        span.set_attribute(KeyValue::new("run_id", trace.run_id.clone()));
        span.set_attribute(KeyValue::new(
            "workflow_name",
            trace.workflow_name.clone(),
        ));
        span.set_attribute(KeyValue::new("status", format!("{:?}", trace.status)));
        for step in &trace.steps {
            tracer.in_span(format!("step/{}", step.step_index), |step_cx| {
                for attr in step_metadata_attrs(step) {
                    step_cx.span().set_attribute(attr);
                }
            });
        }
    });
    Ok(())
}

/// Best-effort async export after a run completes; never blocks the caller.
pub fn maybe_export_trace(run_id: &str) {
    if !otlp_configured() {
        return;
    }
    let Some(trace) = get_execution_trace(run_id) else {
        warn!(run_id, "otlp export skipped: trace not found");
        return;
    };
    let run_id = run_id.to_string();
    std::thread::spawn(move || {
        if let Err(err) = export_trace_blocking(&trace) {
            warn!(run_id = %run_id, error = %err, "otlp export failed");
        }
    });
}
