//! OTel configuration gate (no opentelemetry crate dependency).

const DEFAULT_SERVICE_NAME: &str = "arcflow-runtime";

/// Transport protocol for OTLP span export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtlpProtocol {
    Grpc,
    HttpProtobuf,
}

/// Master switch (`ARCFLOW_OTEL_ENABLED`); default off.
pub fn otel_enabled() -> bool {
    parse_bool_env("ARCFLOW_OTEL_ENABLED").unwrap_or(false)
}

/// True when export was requested via master switch or legacy endpoint.
pub fn export_requested() -> bool {
    otel_enabled() || legacy_otlp_endpoint().is_some()
}

pub(crate) fn legacy_otlp_endpoint() -> Option<String> {
    std::env::var("ARCFLOW_OTLP_ENDPOINT")
        .ok()
        .filter(|v| !v.trim().is_empty())
}

/// Resolved OTLP endpoint (`OTEL_EXPORTER_OTLP_ENDPOINT`, then legacy var).
pub fn otlp_endpoint() -> Option<String> {
    std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(legacy_otlp_endpoint)
}

pub fn otlp_protocol() -> OtlpProtocol {
    match std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL")
        .unwrap_or_else(|_| "http/protobuf".into())
        .trim()
        .to_lowercase()
        .as_str()
    {
        "grpc" => OtlpProtocol::Grpc,
        _ => OtlpProtocol::HttpProtobuf,
    }
}

pub fn service_name() -> String {
    std::env::var("OTEL_SERVICE_NAME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SERVICE_NAME.to_string())
}

pub fn resource_attributes() -> Vec<(String, String)> {
    std::env::var("OTEL_RESOURCE_ATTRIBUTES")
        .ok()
        .map(|raw| parse_resource_attributes(&raw))
        .unwrap_or_default()
}

fn parse_bool_env(key: &str) -> Option<bool> {
    std::env::var(key).ok().map(|v| {
        matches!(
            v.trim().to_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn parse_resource_attributes(raw: &str) -> Vec<(String, String)> {
    raw.split(',')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            let k = k.trim();
            let v = v.trim();
            if k.is_empty() {
                None
            } else {
                Some((k.to_string(), v.to_string()))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_otel_env() {
        for key in [
            "ARCFLOW_OTEL_ENABLED",
            "ARCFLOW_OTLP_ENDPOINT",
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            "OTEL_EXPORTER_OTLP_PROTOCOL",
            "OTEL_SERVICE_NAME",
            "OTEL_RESOURCE_ATTRIBUTES",
        ] {
            std::env::remove_var(key);
        }
    }

    fn with_env(test: impl FnOnce()) {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        clear_otel_env();
        test();
        clear_otel_env();
    }

    #[test]
    fn otel_env_config_matrix() {
        with_env(|| assert!(!export_requested()));

        with_env(|| {
            std::env::set_var("ARCFLOW_OTEL_ENABLED", "true");
            assert!(otel_enabled());
            assert!(export_requested());
        });

        with_env(|| {
            std::env::set_var("ARCFLOW_OTLP_ENDPOINT", "http://localhost:4317");
            assert!(export_requested());
            assert_eq!(
                otlp_endpoint().as_deref(),
                Some("http://localhost:4317")
            );
        });

        with_env(|| {
            std::env::set_var("ARCFLOW_OTLP_ENDPOINT", "http://legacy:4317");
            std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318");
            assert_eq!(
                otlp_endpoint().as_deref(),
                Some("http://collector:4318")
            );
        });

        with_env(|| {
            std::env::set_var(
                "OTEL_RESOURCE_ATTRIBUTES",
                "deployment.environment=prod,service.version=1.0",
            );
            let attrs = resource_attributes();
            assert!(attrs.contains(&(
                "deployment.environment".into(),
                "prod".into()
            )));
            assert!(attrs.contains(&("service.version".into(), "1.0".into())));
        });
    }
}
