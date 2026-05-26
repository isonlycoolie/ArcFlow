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
