//! OpenTelemetry export (optional OTLP). See ADR-009.

/// Returns true when `ARCFLOW_OTLP_ENDPOINT` is set.
pub fn otlp_configured() -> bool {
    std::env::var("ARCFLOW_OTLP_ENDPOINT")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}
