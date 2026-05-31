//! JSON Schema validation for external outcome reports.

use jsonschema::{Draft, Validator};
use serde_json::{json, Value};

use crate::rcs::types::{ExternalBinding, ExternalOutcomeReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeError {
    BindingMismatch { expected: String, got: String },
    SchemaValidation { reason: String },
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BindingMismatch { expected, got } => {
                write!(f, "binding_id mismatch: expected '{expected}', got '{got}'")
            }
            Self::SchemaValidation { reason } => write!(f, "outcome schema validation failed: {reason}"),
        }
    }
}

impl std::error::Error for EnvelopeError {}

/// Validates `report` against the binding's outcome schema and id.
pub fn validate_outcome_envelope(
    binding: &ExternalBinding,
    report: &ExternalOutcomeReport,
) -> Result<(), EnvelopeError> {
    if report.binding_id != binding.id {
        return Err(EnvelopeError::BindingMismatch {
            expected: binding.id.clone(),
            got: report.binding_id.clone(),
        });
    }
    let payload = outcome_to_json(report);
    validate_against_schema(&binding.outcome_schema, &payload)
}

fn outcome_to_json(report: &ExternalOutcomeReport) -> Value {
    let status = match report.status {
        crate::rcs::types::ExternalOutcomeStatus::Success => "success",
        crate::rcs::types::ExternalOutcomeStatus::Failed => "failed",
        crate::rcs::types::ExternalOutcomeStatus::NeedsInput => "needs_input",
    };
    let mut obj = json!({
        "binding_id": report.binding_id,
        "status": status,
    });
    if let Some(code) = &report.error_code {
        obj["error_code"] = json!(code);
    }
    if let Some(fields) = &report.fields {
        obj["fields"] = fields.clone();
    }
    if let Some(refs) = &report.artifact_refs {
        obj["artifact_refs"] = json!(refs);
    }
    obj
}

fn validate_against_schema(schema: &Value, payload: &Value) -> Result<(), EnvelopeError> {
    let validator = Validator::options()
        .with_draft(Draft::Draft7)
        .build(schema)
        .map_err(|e| EnvelopeError::SchemaValidation {
            reason: format!("invalid outcome_schema: {e}"),
        })?;
    if let Err(error) = validator.validate(payload) {
        return Err(EnvelopeError::SchemaValidation {
            reason: error.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rcs::types::{
        ExternalBindingKind, ExternalBindingMode, ExternalOutcomeStatus,
    };
    use serde_json::json;
    use uuid::Uuid;

    fn binding() -> ExternalBinding {
        ExternalBinding {
            id: "gov_portal".into(),
            kind: ExternalBindingKind::BrowserAutomation,
            attach_to_step_id: Uuid::new_v4(),
            mode: ExternalBindingMode::AsyncCallback,
            outcome_schema: json!({
                "type": "object",
                "properties": {
                    "status": { "enum": ["success", "failed", "needs_input"] }
                },
                "required": ["status"]
            }),
            recovery: None,
        }
    }

    #[test]
    fn valid_outcome_passes() {
        let b = binding();
        let report = ExternalOutcomeReport {
            binding_id: "gov_portal".into(),
            status: ExternalOutcomeStatus::Success,
            error_code: None,
            fields: None,
            artifact_refs: None,
        };
        validate_outcome_envelope(&b, &report).unwrap();
    }

    #[test]
    fn binding_id_mismatch_fails() {
        let b = binding();
        let report = ExternalOutcomeReport {
            binding_id: "other".into(),
            status: ExternalOutcomeStatus::Success,
            error_code: None,
            fields: None,
            artifact_refs: None,
        };
        assert!(matches!(
            validate_outcome_envelope(&b, &report),
            Err(EnvelopeError::BindingMismatch { .. })
        ));
    }
}
