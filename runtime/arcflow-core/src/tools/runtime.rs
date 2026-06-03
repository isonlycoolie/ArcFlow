//! Coordinates tool validation, execution, and trace emission.

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use uuid::Uuid;

use crate::tracing::{
    emitter::TraceEmitter, sprint5_emitter::TraceEventEmitter, tool_finished, tool_started,
};

use super::error::ToolError;
use tokio::time::timeout;

use super::executor::{spawn_invoke, ToolInvoker};
use super::registry::{RegisteredTool, ToolRegistry};
use super::validation::validate_tool_input;

/// Tool execution coordinator for one workflow run.
#[derive(Debug)]
pub struct ToolRuntime {
    registry: ToolRegistry,
}

impl Default for ToolRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRuntime {
    /// Empty runtime.
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Registers a tool definition.
    pub fn register(&mut self, tool: RegisteredTool) -> Result<(), ToolError> {
        self.registry.register(tool)
    }

    /// Reference to the underlying registry.
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Whether any tools are registered.
    pub fn has_tools(&self) -> bool {
        self.registry.names().next().is_some()
    }

    /// Validates and executes one tool, emitting trace metadata.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_tool(
        &self,
        name: &str,
        input: Value,
        invoker: Arc<dyn ToolInvoker>,
        legacy: &mut TraceEmitter,
        sprint5: &mut TraceEventEmitter<'_>,
        run_id: &str,
        step_id: Option<Uuid>,
    ) -> Result<String, ToolError> {
        let Some(tool) = self.registry.get(name) else {
            return Err(ToolError::NotRegistered {
                name: name.to_string(),
            });
        };
        validate_tool_input(name, &tool.input_schema, &input)?;
        let step_key = step_id.map(|s| s.to_string()).unwrap_or_default();
        #[cfg(feature = "otel")]
        let _tool_span = crate::tracing::otel_live::tool_span(run_id, &step_key, name);
        if let Some(sid) = step_id {
            tool_started(sprint5, run_id, sid, name);
        }
        let started = std::time::Instant::now();
        let duration = Duration::from_secs(tool.timeout_secs.max(1));
        let fut = spawn_invoke(invoker, name.to_string(), input);
        let result = match timeout(duration, fut).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ToolError::Timeout {
                name: name.to_string(),
                timeout_secs: tool.timeout_secs,
            }),
        };
        let ok = result.is_ok();
        let duration_ms = started.elapsed().as_millis() as u64;
        #[cfg(feature = "otel")]
        crate::tracing::otel_live::record_tool_result(duration_ms, if ok { "ok" } else { "error" });
        let out_len = result.as_ref().map(|s| s.len()).unwrap_or(0);
        tool_finished(
            legacy,
            sprint5,
            run_id,
            step_id,
            name,
            ok,
            duration_ms,
            out_len,
            if ok { None } else { Some("tool_failed") },
        );
        result.map_err(|e| match e {
            ToolError::ExecutionFailed {
                name: n,
                step_id: _,
                reason,
            } => ToolError::ExecutionFailed {
                name: n,
                step_id,
                reason,
            },
            other => other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::store::TraceStore;
    use serde_json::json;

    struct EchoInvoker;

    impl ToolInvoker for EchoInvoker {
        fn invoke(&self, _name: &str, input: &Value) -> Result<String, ToolError> {
            Ok(input
                .get("q")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string())
        }
    }

    #[tokio::test]
    async fn execute_tool_returns_echo() {
        let mut rt = ToolRuntime::new();
        rt.register(RegisteredTool {
            name: "echo".into(),
            input_schema: json!({"type":"object","properties":{"q":{"type":"string"}}}),
            timeout_secs: 5,
        })
        .unwrap();
        let run_id = Uuid::new_v4();
        let mut store = TraceStore::new();
        let run_key = run_id.to_string();
        let mut legacy = TraceEmitter::new(run_id);
        let mut sprint5 = TraceEventEmitter::new(run_key.clone(), &mut store);
        let out = rt
            .execute_tool(
                "echo",
                json!({"q": "hi"}),
                Arc::new(EchoInvoker),
                &mut legacy,
                &mut sprint5,
                &run_key,
                None,
            )
            .await
            .unwrap();
        assert_eq!(out, "hi");
        assert!(!legacy.events().is_empty());
    }
}
