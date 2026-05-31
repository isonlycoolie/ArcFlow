//! External orchestration — LINK/UNDERSTAND/MONITOR/ACT (Phase 2-Pro v2).

mod binding;
mod envelope;
mod monitor;
mod recovery;
pub mod resume;
pub mod webhook;

pub use binding::{find_binding, validate_bindings};
pub use envelope::{validate_outcome_envelope, EnvelopeError};
pub use monitor::{emit_external_traces, MonitorContext};
pub use recovery::{decide_recovery, RecoveryAction, RecoveryDecision};
pub use resume::{resume_workflow_with_external_outcome, EXTERNAL_INTERRUPT_CODE};
pub use webhook::{compute_hmac_sha256_hex, verify_webhook_signature};
