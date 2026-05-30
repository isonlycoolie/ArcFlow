//! Streaming engine (Sprint 6 — runtime-internal) and SDK stream events (Phase 2.1).

pub mod bridge;
pub mod channel;
pub mod engine;
pub mod events;

pub use bridge::{StreamRunBridge, StreamRunOutcome};
pub use channel::{default_stream_pair, stream_pair, StreamChannelSender};
pub use engine::StreamingEngine;
pub use events::StreamEvent;
