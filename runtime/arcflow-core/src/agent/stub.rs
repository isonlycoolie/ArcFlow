//! Stub agent conventions for Sprint 2 (no LLM).

/// When an agent's `role` equals this value, the stub returns [`crate::error::RuntimeError::AgentExecutionFailed`].
pub const STUB_FAIL_ROLE: &str = "__fail__";
