//! PostgreSQL storage for pending human approvals.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::constants::RECOVERY_STORAGE_TIMEOUT_SECS;
use crate::error::RuntimeError;

pub struct HumanApprovalStorage {
    pool: PgPool,
}

impl HumanApprovalStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_pending(
        &self,
        run_id: &str,
        approval_key: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RuntimeError> {
        tokio::time::timeout(
            std::time::Duration::from_secs(RECOVERY_STORAGE_TIMEOUT_SECS),
            sqlx::query(
                "INSERT INTO arcflow_human_approvals (run_id, approval_key, status, expires_at)
                 VALUES ($1, $2, 'pending', $3)
                 ON CONFLICT (run_id, approval_key) DO NOTHING",
            )
            .bind(run_id)
            .bind(approval_key)
            .bind(expires_at)
            .execute(&self.pool),
        )
        .await
        .map_err(|_| RuntimeError::RecoveryStorageError {
            reason: "approval insert timed out".into(),
        })?
        .map_err(|e| RuntimeError::RecoveryStorageError {
            reason: format!("approval insert failed: {e}"),
        })?;
        Ok(())
    }

    pub async fn resolve(
        &self,
        run_id: &str,
        approval_key: &str,
        approved: bool,
        data: &Value,
    ) -> Result<ApprovalResolveOutcome, RuntimeError> {
        use sqlx::Row;
        let row = tokio::time::timeout(
            std::time::Duration::from_secs(RECOVERY_STORAGE_TIMEOUT_SECS),
            sqlx::query(
                "SELECT status, approved, data_json, expires_at
                 FROM arcflow_human_approvals
                 WHERE run_id = $1 AND approval_key = $2",
            )
            .bind(run_id)
            .bind(approval_key)
            .fetch_optional(&self.pool),
        )
        .await
        .map_err(|_| RuntimeError::RecoveryStorageError {
            reason: "approval load timed out".into(),
        })?
        .map_err(|e| RuntimeError::RecoveryStorageError {
            reason: format!("approval load failed: {e}"),
        })?;

        let Some(row) = row else {
            return Err(RuntimeError::ApprovalNotFound {
                approval_key: approval_key.to_string(),
            });
        };

        let status: String = row.get("status");
        let expires_at: DateTime<Utc> = row.get("expires_at");
        if status != "pending" {
            let existing_approved: Option<bool> = row.get("approved");
            let existing_data: Option<Value> = row.get("data_json");
            if existing_approved == Some(approved) && existing_data.as_ref() == Some(data) {
                return Ok(ApprovalResolveOutcome::AlreadyResolved);
            }
            return Err(RuntimeError::AlreadyApproved {
                approval_key: approval_key.to_string(),
            });
        }
        if Utc::now() > expires_at {
            let _ = sqlx::query(
                "UPDATE arcflow_human_approvals SET status = 'expired', resolved_at = NOW()
                 WHERE run_id = $1 AND approval_key = $2",
