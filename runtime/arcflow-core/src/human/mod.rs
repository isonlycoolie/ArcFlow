//! Human-in-the-loop primitives (Phase 1.4).

mod approve;
mod interrupt;
mod storage;
mod wait;

pub use approve::resume_workflow_with_approval;
pub use interrupt::{ApprovalResult, HitlConfig, HUMAN_INTERRUPT_CODE};
pub use storage::HumanApprovalStorage;
pub(crate) use wait::interrupt_for_human;
