//! ArcFlow native tracing engine (Sprint 5).
//!
//! Sprint 4 uses [`emitter::TraceEmitter`] with RCS event types; Sprint 5 adds
//! structured kinds in [`events`] and assembly in [`builder`] without disabling
//! existing workflow runs.

pub mod builder;
pub mod emitter;
pub mod error;
pub mod events;
pub mod otel;
pub mod sprint5_emitter;
pub mod store;
pub mod types;

pub use builder::ExecutionTraceBuilder;
pub use emitter::TraceEmitter;
pub use sprint5_emitter::TraceEventEmitter;
pub use error::TracingError;
pub use events::TraceEventKind;
pub use store::TraceStore;
pub use types::{ExecutionTrace, StepTrace, TokenUsage, TraceEvent};
