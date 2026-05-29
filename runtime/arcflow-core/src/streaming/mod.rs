//! Streaming engine (Sprint 6 — runtime-internal) and SDK stream events (Phase 2.1).

pub mod engine;
pub mod events;

pub use engine::StreamingEngine;
pub use events::StreamEvent;
