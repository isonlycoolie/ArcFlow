//! Workflow orchestration kernel (Sprint 2).

mod context;
mod engine;
mod execution_config;
mod graph;
mod record;
mod run;
mod run_error;
mod test_config;
mod validation;

pub use context::ExecutionContext;
pub use execution_config::ExecutionConfig;
pub use test_config::{resolve_key, TestConfig, TestStubStep};

pub use engine::WorkflowEngine;
pub use record::WorkflowExecutionRecord;
pub(crate) use run::{partial_record, run_one_step, run_sorted_steps, ResumeParams, RunLoop};
pub use run_error::WorkflowRunError;
