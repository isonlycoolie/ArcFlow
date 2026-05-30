//! ArcFlow native tracing engine (Sprint 5).
//!
//! Sprint 4 uses [`emitter::TraceEmitter`] with RCS event types; Sprint 5 adds
//! structured kinds in [`events`] and assembly in [`builder`] without disabling
//! existing workflow runs.

pub mod builder;
pub mod dual;
pub mod emitter;
pub mod error;
pub mod events;
pub mod otel;
pub mod otel_export;
pub mod persistence;
pub mod registry;
pub mod sprint5_emitter;
pub mod store;
pub mod types;

pub use builder::ExecutionTraceBuilder;
pub use dual::{memory_read, memory_write, tokens_consumed, tool_finished, tool_started};
pub use emitter::TraceEmitter;
pub use error::TracingError;
pub use events::TraceEventKind;
pub use persistence::PostgresTracePersistence;
pub use registry::{get_execution_trace, set_trace_event_persist_hook, try_get_execution_trace, with_store};
pub use sprint5_emitter::TraceEventEmitter;
pub use store::TraceStore;
pub use types::{ExecutionTrace, StepTrace, TokenUsage, TraceEvent};
