//! Site create, read, and token verification.

use super::{map_site, SiteRecord, SiteStore};

pub async fn create(
    store: &SiteStore,
    display_name: &str,
    allowed_origins: &[String],
    rate_limit_rpm: i32,
    allow_inline: bool,
    default_workflow_name: &str,
    upstream_runtime_key: &str,
    chat_instructions: Option<&str>,
) -> Result<(SiteRecord, String), sqlx::Error> {
    let site_id = SiteStore::generate_site_id();
    let token = SiteStore::generate_token();
    let token_hash = SiteStore::hash_token(&token);
    let kb_namespace = format!("site-{site_id}-kb");

    let row = sqlx::query(
        "INSERT INTO arcflow_sites
         (id, display_name, allowed_origins, rate_limit_rpm, allow_inline,
          default_workflow_name, kb_namespace, upstream_runtime_key, chat_instructions)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, display_name, allowed_origins, rate_limit_rpm, allow_inline,
                   default_workflow_name, kb_namespace, upstream_runtime_key,
                   chat_instructions, created_at",
    )
    .bind(&site_id)
    .bind(display_name)
    .bind(allowed_origins)
    .bind(rate_limit_rpm)
    .bind(allow_inline)
    .bind(default_workflow_name)
    .bind(&kb_namespace)
    .bind(upstream_runtime_key)
    .bind(chat_instructions)
    .fetch_one(&store.pool)
    .await?;

    sqlx::query(
        "INSERT INTO arcflow_site_tokens (site_id, token_hash, label)
         VALUES ($1, $2, 'primary')",
    )
    .bind(&site_id)
    .bind(&token_hash)
    .execute(&store.pool)
    .await?;

    Ok((map_site(&row), token))
}

pub async fn get(store: &SiteStore, site_id: &str) -> Result<Option<SiteRecord>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, display_name, allowed_origins, rate_limit_rpm, allow_inline,
                default_workflow_name, kb_namespace, upstream_runtime_key,
                chat_instructions, created_at
         FROM arcflow_sites WHERE id = $1",
    )
    .bind(site_id)
    .fetch_optional(&store.pool)
    .await?;
    Ok(row.map(|r| map_site(&r)))
}

pub async fn verify_token(
    store: &SiteStore,
    site_id: &str,
    token: &str,
) -> Result<Option<SiteRecord>, sqlx::Error> {
    let site = match get(store, site_id).await? {
        Some(s) => s,
        None => return Ok(None),
    };
    let hash = SiteStore::hash_token(token);
    let row = sqlx::query(
        "SELECT 1 FROM arcflow_site_tokens
         WHERE site_id = $1 AND token_hash = $2 AND revoked_at IS NULL",
    )
    .bind(site_id)
    .bind(&hash)
    .fetch_optional(&store.pool)
    .await?;
    if row.is_some() {
        return Ok(Some(site));
    }
    Ok(None)
}
