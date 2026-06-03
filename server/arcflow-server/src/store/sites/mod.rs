//! PostgreSQL persistence for ArcFlow Relay sites.

mod maintenance;
mod persistence;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

pub use maintenance::{origin_allowed, tokens_match};
pub use persistence::{create, get, verify_token};

#[derive(Debug, Clone)]
pub struct SiteRecord {
    pub id: String,
    pub display_name: String,
    pub allowed_origins: Vec<String>,
    pub rate_limit_rpm: i32,
    pub allow_inline: bool,
    pub default_workflow_name: Option<String>,
    pub kb_namespace: String,
    pub upstream_runtime_key: String,
    pub chat_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct SiteStore {
    pub(crate) pool: PgPool,
}

impl SiteStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn hash_token(token: &str) -> String {
        format!("{:x}", Sha256::digest(token.as_bytes()))
    }

    pub fn generate_site_id() -> String {
        format!("s_{}", &Uuid::new_v4().simple().to_string()[..12])
    }

    pub fn generate_token() -> String {
        format!("st_live_{}", &Uuid::new_v4().simple().to_string())
    }

    pub fn origin_allowed(site: &SiteRecord, origin: &str) -> bool {
        maintenance::origin_allowed(site, origin)
    }

    pub async fn create(
        &self,
        display_name: &str,
        allowed_origins: &[String],
        rate_limit_rpm: i32,
        allow_inline: bool,
        default_workflow_name: &str,
        upstream_runtime_key: &str,
        chat_instructions: Option<&str>,
    ) -> Result<(SiteRecord, String), sqlx::Error> {
        persistence::create(
            self,
            display_name,
            allowed_origins,
            rate_limit_rpm,
            allow_inline,
            default_workflow_name,
            upstream_runtime_key,
            chat_instructions,
        )
        .await
    }

    pub async fn get(&self, site_id: &str) -> Result<Option<SiteRecord>, sqlx::Error> {
        persistence::get(self, site_id).await
    }

    pub async fn verify_token(
        &self,
        site_id: &str,
        token: &str,
    ) -> Result<Option<SiteRecord>, sqlx::Error> {
        persistence::verify_token(self, site_id, token).await
    }

    pub async fn rotate_token(&self, site_id: &str) -> Result<Option<String>, sqlx::Error> {
        maintenance::rotate_token(self, site_id).await
    }

    pub async fn patch(
        &self,
        site_id: &str,
        display_name: Option<&str>,
        allowed_origins: Option<&[String]>,
        rate_limit_rpm: Option<i32>,
        allow_inline: Option<bool>,
        chat_instructions: Option<Option<&str>>,
    ) -> Result<Option<SiteRecord>, sqlx::Error> {
        maintenance::patch(
            self,
            site_id,
            display_name,
            allowed_origins,
            rate_limit_rpm,
            allow_inline,
            chat_instructions,
        )
        .await
    }

    pub async fn increment_usage(&self, site_id: &str) -> Result<(), sqlx::Error> {
        maintenance::increment_usage(self, site_id).await
    }
}

pub(crate) fn map_site(row: &sqlx::postgres::PgRow) -> SiteRecord {
    use sqlx::Row;
    SiteRecord {
        id: row.get("id"),
        display_name: row.get("display_name"),
        allowed_origins: row.get("allowed_origins"),
        rate_limit_rpm: row.get("rate_limit_rpm"),
        allow_inline: row.get("allow_inline"),
        default_workflow_name: row.get("default_workflow_name"),
        kb_namespace: row.get("kb_namespace"),
        upstream_runtime_key: row.get("upstream_runtime_key"),
        chat_instructions: row.get("chat_instructions"),
        created_at: row.get("created_at"),
    }
}
