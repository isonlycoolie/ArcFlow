//! HITL types and constants.

pub use crate::rcs::types::{ApprovalResult, HitlConfig};

/// Recovery `failure_error_code` value for intentional human interrupts.
pub const HUMAN_INTERRUPT_CODE: &str = "HumanInterrupt";
