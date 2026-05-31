//! Site updates, token rotation, and usage tracking.

use chrono::Utc;
use subtle::ConstantTimeEq;

use super::{map_site, SiteRecord, SiteStore};

pub fn origin_allowed(site: &SiteRecord, origin: &str) -> bool {
    if origin.is_empty() {
        return false;
    }
    site.allowed_origins.iter().any(|o| o == origin)
}

pub async fn rotate_token(store: &SiteStore, site_id: &str) -> Result<Option<String>, sqlx::Error> {
    if super::persistence::get(store, site_id).await?.is_none() {
        return Ok(None);
    }
    sqlx::query(
        "UPDATE arcflow_site_tokens SET revoked_at = NOW()
         WHERE site_id = $1 AND revoked_at IS NULL",
    )
    .bind(site_id)
    .execute(&store.pool)
    .await?;

    let token = SiteStore::generate_token();
    let token_hash = SiteStore::hash_token(&token);
    sqlx::query(
        "INSERT INTO arcflow_site_tokens (site_id, token_hash, label)
         VALUES ($1, $2, 'primary')",
    )
    .bind(site_id)
    .bind(&token_hash)
    .execute(&store.pool)
    .await?;
    Ok(Some(token))
}

pub async fn patch(
    store: &SiteStore,
    site_id: &str,
    display_name: Option<&str>,
    allowed_origins: Option<&[String]>,
    rate_limit_rpm: Option<i32>,
    allow_inline: Option<bool>,
    chat_instructions: Option<Option<&str>>,
) -> Result<Option<SiteRecord>, sqlx::Error> {
    let current = match super::persistence::get(store, site_id).await? {
        Some(s) => s,
        None => return Ok(None),
    };
    let name = display_name.unwrap_or(&current.display_name);
    let origins = allowed_origins.unwrap_or(&current.allowed_origins);
    let rpm = rate_limit_rpm.unwrap_or(current.rate_limit_rpm);
    let inline = allow_inline.unwrap_or(current.allow_inline);
    let instructions = match chat_instructions {
        Some(v) => v.map(str::to_string),
        None => current.chat_instructions.clone(),
    };

    let row = sqlx::query(
        "UPDATE arcflow_sites SET
            display_name = $2, allowed_origins = $3, rate_limit_rpm = $4,
            allow_inline = $5, chat_instructions = $6
         WHERE id = $1
         RETURNING id, display_name, allowed_origins, rate_limit_rpm, allow_inline,
                   default_workflow_name, kb_namespace, upstream_runtime_key,
                   chat_instructions, created_at",
    )
    .bind(site_id)
    .bind(name)
    .bind(origins)
    .bind(rpm)
    .bind(inline)
    .bind(instructions)
    .fetch_one(&store.pool)
    .await?;
    Ok(Some(map_site(&row)))
}

pub async fn increment_usage(store: &SiteStore, site_id: &str) -> Result<(), sqlx::Error> {
    let today = Utc::now().date_naive();
    sqlx::query(
        "INSERT INTO arcflow_site_usage_daily (site_id, date, run_count)
         VALUES ($1, $2, 1)
         ON CONFLICT (site_id, date) DO UPDATE
         SET run_count = arcflow_site_usage_daily.run_count + 1",
    )
    .bind(site_id)
    .bind(today)
    .execute(&store.pool)
    .await?;
    Ok(())
}

pub fn tokens_match(stored_hash: &str, token: &str) -> bool {
    let computed = SiteStore::hash_token(token);
    computed.as_bytes().ct_eq(stored_hash.as_bytes()).into()
}
