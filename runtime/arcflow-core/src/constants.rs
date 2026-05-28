//! Named runtime limits (Sprint 5 observability and shared caps).

/// Maximum trace events retained per workflow run before dropping oldest events.
pub const MAX_TRACE_EVENTS_PER_RUN: u32 = 10_000;

/// Maximum completed run traces held in the in-process store at once.
pub const MAX_CONCURRENT_TRACES: usize = 100;
