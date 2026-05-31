//! Agent runtime — stub execution in Sprint 2.

mod context;
mod runtime;
mod stub;
mod tool_loop;

pub use context::{ContextAssembler, ContextExtras};
pub use runtime::AgentRuntime;
pub use stub::STUB_FAIL_ROLE;
pub use tool_loop::ToolLoop;
