//! Public OTLP export entry point (works with or without `otel` feature).

use super::otel_config;

/// Best-effort async export after a run completes; never blocks the caller.
pub fn maybe_export_trace(run_id: &str) {
    if !otel_config::export_requested() {
        return;
    }
    #[cfg(feature = "otel")]
    super::otel_export_impl::maybe_export_trace_impl(run_id);
    #[cfg(not(feature = "otel"))]
    let _ = run_id;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_skipped_when_otlp_unconfigured() {
        std::env::remove_var("ARCFLOW_OTLP_ENDPOINT");
        maybe_export_trace("missing-run");
    }
}
