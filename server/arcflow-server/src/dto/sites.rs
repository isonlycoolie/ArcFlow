//! Site admin API request/response types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateSiteRequest {
    pub display_name: String,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_rpm: i32,
    #[serde(default)]
    pub allow_inline: bool,
    #[serde(default = "default_workflow")]
    pub default_workflow_name: String,
    pub upstream_runtime_key: Option<String>,
    pub chat_instructions: Option<String>,
}

fn default_rate_limit() -> i32 {
    60
}

fn default_workflow() -> String {
    "chat".into()
}

#[derive(Debug, Serialize)]
pub struct CreateSiteResponse {
    pub site_id: String,
    pub relay_url: String,
    pub site_token: String,
    pub kb_namespace: String,
}

#[derive(Debug, Serialize)]
pub struct SiteResponse {
    pub site_id: String,
    pub display_name: String,
    pub allowed_origins: Vec<String>,
    pub rate_limit_rpm: i32,
    pub allow_inline: bool,
    pub default_workflow_name: Option<String>,
    pub kb_namespace: String,
    pub chat_instructions: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchSiteRequest {
    pub display_name: Option<String>,
    pub allowed_origins: Option<Vec<String>>,
    pub rate_limit_rpm: Option<i32>,
    pub allow_inline: Option<bool>,
    pub chat_instructions: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RotateTokenResponse {
    pub site_token: String,
}

#[derive(Debug, Deserialize)]
pub struct IngestKnowledgeRequest {
    pub text: String,
    pub key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IngestKnowledgeResponse {
    pub chunks_ingested: usize,
    pub namespace: String,
}

#[derive(Debug, Deserialize)]
pub struct PublishChatRequest {
    pub instructions: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishChatResponse {
    pub name: String,
    pub version: String,
    pub schema_hash: String,
}
