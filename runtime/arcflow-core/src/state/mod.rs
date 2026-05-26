//! Execution state management — one [`StateEngine`] per workflow run (Sprint 2).

mod engine;

pub use engine::{ExecutionStepOutput, StateEngine, StateSnapshot};
