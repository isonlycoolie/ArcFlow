//! PostgreSQL persistence for `arcflow_runs`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use arcflow_core::rcs::types::ExecutionStatus;

use crate::dto::runs::StoredRun;

pub struct RunStore {
    pool: PgPool,
}

impl RunStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_pending(
        &self,
        run_id: &str,
        trace_id: &str,
        workflow_hash: &str,
        exec_config: Option<serde_json::Value>,
        idempotency_key: Option<&str>,
        workflow_json: Option<serde_json::Value>,
        agents_json: Option<serde_json::Value>,
        input_text: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO arcflow_runs
             (run_id, trace_id, status, workflow_hash, exec_config_json, idempotency_key,
              workflow_json, agents_json, input_text, started_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())",
        )
        .bind(run_id)
        .bind(trace_id)
        .bind(status_str(ExecutionStatus::Pending))
        .bind(workflow_hash)
        .bind(exec_config)
        .bind(idempotency_key)
        .bind(workflow_json)
        .bind(agents_json)
        .bind(input_text)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_running(&self, run_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE arcflow_runs SET status = $2, started_at = NOW() WHERE run_id = $1")
            .bind(run_id)
            .bind(status_str(ExecutionStatus::Running))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn mark_completed(
        &self,
        run_id: &str,
        status: ExecutionStatus,
        result_json: Option<serde_json::Value>,
        error_json: Option<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE arcflow_runs
             SET status = $2, result_json = $3, error_json = $4, completed_at = NOW()
             WHERE run_id = $1",
        )
        .bind(run_id)
        .bind(status_str(status))
        .bind(result_json)
        .bind(error_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get(&self, run_id: &str) -> Result<Option<StoredRun>, sqlx::Error> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT run_id, trace_id, status, result_json, error_json, created_at, completed_at,
                    workflow_json, agents_json, input_text, exec_config_json
             FROM arcflow_runs WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| StoredRun {
            run_id: r.get("run_id"),
            trace_id: r.get("trace_id"),
            status: parse_status(r.get::<String, _>("status")),
            result_json: r.get("result_json"),
            error_json: r.get("error_json"),
            created_at: r.get::<DateTime<Utc>, _>("created_at"),
            completed_at: r.get("completed_at"),
            workflow_json: r.get("workflow_json"),
            agents_json: r.get("agents_json"),
            input_text: r.get("input_text"),
