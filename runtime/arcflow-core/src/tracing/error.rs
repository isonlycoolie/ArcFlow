//! Tracing subsystem errors (storage and export only — never workflow execution).

use thiserror::Error;

/// Failures in trace storage or OTLP export. Never surfaced as workflow errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TracingError {
    /// Run id not found in the in-process store.
    #[error("trace not found for run_id '{run_id}'")]
    TraceNotFound { run_id: String },

    /// OTLP export failed after workflow completed.
    #[error("otlp export failed: {reason}")]
    OtlpExportFailed { reason: String },
}
