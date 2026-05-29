//! Request/response DTOs for approval endpoint.

use serde::{Deserialize, Serialize};

use arcflow_core::rcs::types::ExecutionStatus;

#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    pub approved: bool,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ApproveResponse {
    pub status: ExecutionStatus,
    pub message: String,
}
