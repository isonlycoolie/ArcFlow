use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExternalCallbackResponse {
    pub run_id: String,
    pub binding_id: String,
    pub status: String,
}
