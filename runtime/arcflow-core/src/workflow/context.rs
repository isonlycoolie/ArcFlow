//! Optional Sprint 4 execution context (tools + memory + trace).

use std::sync::Arc;
use std::time::Instant;

use crate::memory::MemoryCoordinator;
use crate::providers::ModelProvider;
use crate::retry::RetryConfig;
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
    /// Provider generation limits when a provider is active.
    pub provider_max_tokens: u32,
    pub provider_temperature: f32,
    /// Workflow-level retry configuration (Sprint 7).
    pub retry_config: Option<RetryConfig>,
    /// Per-step wall-clock limit (Sprint 7).
    pub step_timeout: Option<std::time::Duration>,
    /// Absolute deadline for the whole workflow run (Sprint 7).
    pub workflow_deadline: Option<Instant>,
}
