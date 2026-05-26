//! Runtime Contract Specification — protocol errors at the SDK boundary.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur while handling RCS messages.
///
/// Variant names and serialized shapes are part of the public contract within a
/// major RCS version.
#[derive(Debug, Clone, Error, Serialize, Deserialize, PartialEq, Eq)]
pub enum RcsError {
    /// The envelope `rcs_version` is not supported by this runtime.
    #[error("unsupported RCS version: {version}")]
    UnsupportedVersion {
        /// Version string from the envelope.
        version: String,
    },

    /// A mandatory envelope field was absent or empty.
    #[error("missing required envelope field: {field}")]
    MissingEnvelopeField {
        /// Name of the missing field.
        field: String,
    },

    /// `message_type` could not be mapped to a known variant.
    #[error("invalid message type: {message_type}")]
    InvalidMessageType {
        /// Raw message type string from the envelope.
        message_type: String,
    },

    /// Typed payload could not be deserialized from the envelope body.
    #[error("payload deserialization failed: {reason}")]
    PayloadDeserializationFailed {
        /// Human-readable parse or schema reason.
        reason: String,
    },

    /// Workflow definition failed structural validation.
    #[error("workflow definition invalid: {reason}")]
    InvalidWorkflowDefinition {
        /// Validation failure detail.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_version_error_is_descriptive() {
        let error = RcsError::UnsupportedVersion {
            version: "99.0".to_string(),
        };
        assert!(error.to_string().contains("99.0"));
    }

    #[test]
    fn missing_envelope_field_error_is_descriptive() {
        let error = RcsError::MissingEnvelopeField {
            field: "trace_id".to_string(),
        };
        assert!(error.to_string().contains("trace_id"));
    }

    #[test]
    fn invalid_message_type_error_is_descriptive() {
        let error = RcsError::InvalidMessageType {
            message_type: "Unknown".to_string(),
        };
        assert!(error.to_string().contains("Unknown"));
    }

    #[test]
    fn payload_deserialization_failed_error_is_descriptive() {
        let error = RcsError::PayloadDeserializationFailed {
            reason: "bad json".to_string(),
        };
        assert!(error.to_string().contains("bad json"));
    }

    #[test]
    fn invalid_workflow_definition_error_is_descriptive() {
        let error = RcsError::InvalidWorkflowDefinition {
            reason: "empty steps".to_string(),
        };
        assert!(error.to_string().contains("empty steps"));
    }
}
