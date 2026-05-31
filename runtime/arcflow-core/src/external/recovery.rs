//! Recovery action mapping for external outcomes.

use crate::rcs::types::{
    ExternalFatalAction, ExternalNeedsInputAction, ExternalOutcomeReport, ExternalOutcomeStatus,
    ExternalRecoveryPolicy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    ResumeSuccess,
    RetryExternal,
    InjectToolResult,
    RequestHitl,
    FailRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryDecision {
    pub action: RecoveryAction,
    pub attempt_number: u32,
}

/// Maps an external outcome and retry attempt to a recovery action.
pub fn decide_recovery(
    report: &ExternalOutcomeReport,
    policy: &ExternalRecoveryPolicy,
    attempt_number: u32,
) -> RecoveryDecision {
    let action = match report.status {
        ExternalOutcomeStatus::Success => RecoveryAction::ResumeSuccess,
        ExternalOutcomeStatus::NeedsInput => match policy.on_needs_input {
            ExternalNeedsInputAction::AgentReask => RecoveryAction::InjectToolResult,
            ExternalNeedsInputAction::FailRun => RecoveryAction::FailRun,
        },
        ExternalOutcomeStatus::Failed => {
            if attempt_number <= policy.max_retries {
                RecoveryAction::RetryExternal
            } else {
                match policy.on_fatal {
                    ExternalFatalAction::HitlEscalate => RecoveryAction::RequestHitl,
                    ExternalFatalAction::FailRun => RecoveryAction::FailRun,
                }
            }
        }
    };
    RecoveryDecision {
        action,
        attempt_number,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rcs::types::ExternalOutcomeStatus;

    fn policy(max_retries: u32) -> ExternalRecoveryPolicy {
        ExternalRecoveryPolicy {
            max_retries,
            on_needs_input: ExternalNeedsInputAction::AgentReask,
            on_fatal: ExternalFatalAction::FailRun,
        }
    }

    #[test]
    fn success_resumes() {
        let report = ExternalOutcomeReport {
            binding_id: "b".into(),
            status: ExternalOutcomeStatus::Success,
            error_code: None,
            fields: None,
            artifact_refs: None,
        };
        let d = decide_recovery(&report, &policy(0), 0);
        assert_eq!(d.action, RecoveryAction::ResumeSuccess);
    }

    #[test]
    fn failed_retries_while_budget_remains() {
        let report = ExternalOutcomeReport {
            binding_id: "b".into(),
            status: ExternalOutcomeStatus::Failed,
            error_code: Some("TIMEOUT".into()),
            fields: None,
            artifact_refs: None,
        };
        let d = decide_recovery(&report, &policy(2), 1);
        assert_eq!(d.action, RecoveryAction::RetryExternal);
    }

    #[test]
    fn needs_input_triggers_reask() {
        let report = ExternalOutcomeReport {
            binding_id: "b".into(),
            status: ExternalOutcomeStatus::NeedsInput,
            error_code: Some("INVALID_NAME".into()),
            fields: None,
            artifact_refs: None,
        };
        let d = decide_recovery(&report, &policy(0), 0);
        assert_eq!(d.action, RecoveryAction::InjectToolResult);
    }
}
