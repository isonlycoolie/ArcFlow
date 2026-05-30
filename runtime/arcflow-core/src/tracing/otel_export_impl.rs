//! OTLP span export implementation (requires `otel` feature).

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
        span.set_attribute(KeyValue::new("workflow_name", trace.workflow_name.clone()));
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

pub fn maybe_export_trace_impl(run_id: &str) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::types::{StepExecutionStatus, TokenUsage};

    fn sample_step(role: &str) -> StepTrace {
        StepTrace {
            step_index: 0,
            step_id: "step-0".into(),
            agent_name: "agent".into(),
            agent_role: role.into(),
            status: StepExecutionStatus::Completed,
            started_at: chrono::Utc::now(),
            completed_at: None,
            duration_ms: Some(1),
            tokens: TokenUsage::default(),
            tool_calls: Vec::new(),
            memory_operations: Vec::new(),
            error: None,
        }
    }

    #[test]
    fn step_metadata_attrs_use_schema_fields_only() {
        let secret = "super-secret-instructions";
        let attrs = step_metadata_attrs(&sample_step("researcher"));
        let blob: String = attrs
            .iter()
            .map(|kv| format!("{:?}", kv))
            .collect::<Vec<_>>()
            .join("");
        assert!(blob.contains("step_id"));
        assert!(!blob.contains(secret));
    }
}
