//! Optional Sprint 4 execution context (tools + memory + trace).

use std::sync::Arc;

use crate::memory::MemoryCoordinator;
use crate::providers::ModelProvider;
use crate::tools::{ToolInvoker, ToolRuntime};
use crate::tracing::{emitter::TraceEmitter, sprint5_emitter::TraceEventEmitter};

/// Per-run resources for tools, memory, and trace emission.
pub struct ExecutionContext<'a, 's> {
    /// Registered tools for this run.
    pub tool_runtime: Option<&'a ToolRuntime>,
    /// Invoker implementation (Python binding or test double).
    pub tool_invoker: Option<Arc<dyn ToolInvoker>>,
    /// Memory coordinator for this run.
    pub memory: &'a mut MemoryCoordinator,
    /// Sprint 4 RCS trace events.
    pub legacy: &'a mut TraceEmitter,
    /// Sprint 5 structured trace emitter.
    pub sprint5: &'a mut TraceEventEmitter<'s>,
    /// Run id for Sprint 5 payloads.
    pub run_id: String,
    /// Optional LLM provider for this run (Sprint 6).
    pub provider: Option<Arc<dyn ModelProvider>>,
}
