//! OTel configuration gate (no opentelemetry crate dependency).

/// True when export was requested via env (legacy endpoint check; expanded in env-config branch).
pub fn export_requested() -> bool {
    legacy_otlp_endpoint().is_some()
}

pub(crate) fn legacy_otlp_endpoint() -> Option<String> {
    std::env::var("ARCFLOW_OTLP_ENDPOINT")
        .ok()
        .filter(|v| !v.trim().is_empty())
}
