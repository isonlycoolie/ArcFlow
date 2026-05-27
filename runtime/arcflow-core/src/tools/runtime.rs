//! Coordinates tool validation, execution, and trace emission.

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use uuid::Uuid;

use crate::tracing::TraceEmitter;

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
    pub async fn execute_tool(
        &self,
        name: &str,
        input: Value,
        invoker: Arc<dyn ToolInvoker>,
        trace: &mut TraceEmitter,
        step_id: Option<Uuid>,
    ) -> Result<String, ToolError> {
        let Some(tool) = self.registry.get(name) else {
            return Err(ToolError::NotRegistered {
                name: name.to_string(),
            });
        };
        validate_tool_input(name, &tool.input_schema, &input)?;
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
        let status = if result.is_ok() { "ok" } else { "failed" };
        trace.tool_executed(step_id, name, status, started.elapsed().as_millis() as u64);
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
    use crate::tracing::TraceEmitter;
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
        let mut trace = TraceEmitter::new(Uuid::new_v4());
        let out = rt
            .execute_tool(
                "echo",
                json!({"q": "hi"}),
                Arc::new(EchoInvoker),
                &mut trace,
                None,
            )
            .await
            .unwrap();
        assert_eq!(out, "hi");
        assert!(!trace.events().is_empty());
    }
}
