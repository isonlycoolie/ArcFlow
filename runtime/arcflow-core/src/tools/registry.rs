//! Tool registration for a workflow run.

use std::collections::HashMap;

use serde_json::Value;

use super::error::ToolError;

/// Registered tool metadata and per-tool timeout.
#[derive(Debug, Clone)]
pub struct RegisteredTool {
    /// Tool name used for dispatch.
    pub name: String,
    /// JSON Schema for inputs.
    pub input_schema: Value,
    /// Wall-clock timeout in whole seconds.
    pub timeout_secs: u64,
}

/// In-memory registry for one workflow execution.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, RegisteredTool>,
}

impl ToolRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a tool or returns [`ToolError::DuplicateName`].
    pub fn register(&mut self, tool: RegisteredTool) -> Result<(), ToolError> {
        if self.tools.contains_key(&tool.name) {
            return Err(ToolError::DuplicateName {
                name: tool.name.clone(),
            });
        }
        self.tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    /// Looks up a registered tool.
    pub fn get(&self, name: &str) -> Option<&RegisteredTool> {
        self.tools.get(name)
    }

    /// All registered tool names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.tools.keys().map(String::as_str)
    }
}
