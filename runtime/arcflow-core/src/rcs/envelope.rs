//! Runtime Contract Specification — message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::types::MessageType;

/// Wire-format wrapper for every RCS message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Protocol version string (`MAJOR.MINOR`).
    pub rcs_version: String,
    /// Dispatch label for the payload type.
    pub message_type: MessageType,
    /// Correlation id for traces and logs.
    pub trace_id: Uuid,
    /// UTC timestamp when the message was created.
    pub timestamp: DateTime<Utc>,
    /// Typed payload serialized as JSON.
    pub payload: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn message_envelope_round_trip() {
        let original = MessageEnvelope {
            rcs_version: "1.0".to_string(),
            message_type: MessageType::RunWorkflow,
            trace_id: Uuid::new_v4(),
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            payload: serde_json::json!({"workflow_id": Uuid::new_v4()}),
        };
        let json = serde_json::to_string(&original).expect("MessageEnvelope must serialize");
        let deserialized: MessageEnvelope =
            serde_json::from_str(&json).expect("MessageEnvelope must deserialize");
        assert_eq!(original, deserialized);
    }
}
