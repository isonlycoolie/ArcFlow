//! Optional Sprint 4 execution context (tools + memory + trace).

use std::sync::Arc;

use crate::memory::MemoryCoordinator;
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::tracing::TraceEmitter;

/// Per-run resources for tools, memory, and trace emission.
pub struct ExecutionContext<'a> {
    /// Registered tools for this run.
    pub tool_runtime: Option<&'a ToolRuntime>,
    /// Invoker implementation (Python binding or test double).
    pub tool_invoker: Option<Arc<dyn ToolInvoker>>,
    /// Memory coordinator for this run.
    pub memory: &'a mut MemoryCoordinator,
    /// Trace collector.
    pub trace: &'a mut TraceEmitter,
}
