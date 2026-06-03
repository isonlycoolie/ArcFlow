//! Emit SEC-1 compliant trace events for external bindings.

use crate::rcs::types::{ExternalBinding, ExternalOutcomeReport, ExternalOutcomeStatus};
use crate::tracing::events::TraceEventKind;
use crate::tracing::{with_store, TraceEventEmitter};

use super::recovery::{RecoveryAction, RecoveryDecision};

pub struct MonitorContext {
    pub run_id: String,
    pub step_id: String,
    pub started_at_ms: u64,
}

/// Emits binding lifecycle traces and recovery trace for an outcome.
pub fn emit_external_traces(
    ctx: &MonitorContext,
    binding: &ExternalBinding,
    report: &ExternalOutcomeReport,
    decision: &RecoveryDecision,
) {
    let _ = with_store(|store| {
        let mut emitter = TraceEventEmitter::new(ctx.run_id.clone(), store);
        let mode = match binding.mode {
            crate::rcs::types::ExternalBindingMode::SyncTool => "sync_tool",
            crate::rcs::types::ExternalBindingMode::AsyncCallback => "async_callback",
        };
        emitter.emit(TraceEventKind::ExternalBindingStarted {
            run_id: ctx.run_id.clone(),
            binding_id: binding.id.clone(),
            step_id: ctx.step_id.clone(),
            mode: mode.into(),
        });

        match report.status {
            ExternalOutcomeStatus::Success => {
                let duration_ms = now_ms().saturating_sub(ctx.started_at_ms);
                emitter.emit(TraceEventKind::ExternalBindingCompleted {
                    run_id: ctx.run_id.clone(),
                    binding_id: binding.id.clone(),
                    step_id: ctx.step_id.clone(),
                    duration_ms,
                });
            }
            ExternalOutcomeStatus::Failed | ExternalOutcomeStatus::NeedsInput => {
                let status = match report.status {
                    ExternalOutcomeStatus::Failed => "failed",
                    ExternalOutcomeStatus::NeedsInput => "needs_input",
                    ExternalOutcomeStatus::Success => "success",
                };
                emitter.emit(TraceEventKind::ExternalBindingFailed {
                    run_id: ctx.run_id.clone(),
                    binding_id: binding.id.clone(),
                    step_id: ctx.step_id.clone(),
                    error_code: report
                        .error_code
                        .clone()
                        .unwrap_or_else(|| "UNKNOWN".into()),
                    status: status.into(),
                });
            }
        }

        if decision.action != RecoveryAction::ResumeSuccess {
            let action = match decision.action {
                RecoveryAction::ResumeSuccess => "resume_success",
                RecoveryAction::RetryExternal => "retry_external",
                RecoveryAction::InjectToolResult => "inject_tool_result",
                RecoveryAction::RequestHitl => "request_hitl",
                RecoveryAction::FailRun => "fail_run",
            };
            emitter.emit(TraceEventKind::ExternalRecoveryTriggered {
                run_id: ctx.run_id.clone(),
                binding_id: binding.id.clone(),
                action: action.into(),
                attempt_number: decision.attempt_number,
            });
        }
    });
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
