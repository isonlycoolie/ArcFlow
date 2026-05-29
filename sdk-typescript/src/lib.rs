//! napi-rs bindings for ArcFlow — translate only; orchestration lives in `arcflow_core`.

mod errors;
mod types;
mod workflow;

pub use workflow::{execute_workflow, get_execution_trace_json, get_version, JsWorkflowResult};
