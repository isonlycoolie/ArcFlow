//! Authorization checks for static runtime keys.

use axum::http::StatusCode;

use crate::middleware::{workflow_allowed, AuthPrincipal};

pub fn deny_publish(principal: &AuthPrincipal) -> Result<(), (StatusCode, String)> {
    if let AuthPrincipal::Runtime(policy) = principal {
        if !policy.publish {
            return Err((
                StatusCode::FORBIDDEN,
                "[ArcFlow] Runtime key cannot publish workflows.".into(),
            ));
        }
    }
    Ok(())
}

pub fn ensure_run_workflow(
    principal: &AuthPrincipal,
    workflow_name: &str,
) -> Result<(), (StatusCode, String)> {
    if let AuthPrincipal::Runtime(policy) = principal {
        if !workflow_allowed(policy, workflow_name) {
            return Err((
                StatusCode::FORBIDDEN,
                format!("[ArcFlow] Runtime key is not allowed to run workflow '{workflow_name}'."),
            ));
        }
    }
    Ok(())
}
