//! Postgres persistence for workflow trace events (Phase 1 closure).

use sqlx::PgPool;

use super::builder::ExecutionTraceBuilder;
use super::types::{ExecutionTrace, TraceEvent};

/// Persists and loads redacted trace events keyed by run id.
pub struct PostgresTracePersistence {
    pool: PgPool,
}

impl PostgresTracePersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Inserts events for a run (idempotent per seq via ON CONFLICT DO NOTHING).
    pub async fn persist_events(
        &self,
        run_id: &str,
        events: &[TraceEvent],
    ) -> Result<(), sqlx::Error> {
        for event in events {
            let json = serde_json::to_value(event).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            sqlx::query(
                "INSERT INTO arcflow_trace_events (run_id, seq, event_json)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (run_id, seq) DO NOTHING",
            )
            .bind(run_id)
            .bind(event.sequence as i64)
            .bind(json)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Loads events in sequence order for a run.
    pub async fn load_events(&self, run_id: &str) -> Result<Vec<TraceEvent>, sqlx::Error> {
        let rows: Vec<(i64, serde_json::Value)> = sqlx::query_as(
            "SELECT seq, event_json FROM arcflow_trace_events
             WHERE run_id = $1 ORDER BY seq ASC",
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for (_seq, json) in rows {
            let event: TraceEvent = serde_json::from_value(json)
                .map_err(|e| sqlx::Error::Protocol(format!("invalid trace event json: {e}")))?;
            events.push(event);
        }
        Ok(events)
    }

    /// Builds an execution trace from persisted events.
    pub async fn load_execution_trace(&self, run_id: &str) -> Result<Option<ExecutionTrace>, sqlx::Error> {
        let events = self.load_events(run_id).await?;
        if events.is_empty() {
            return Ok(None);
        }
        Ok(Some(ExecutionTraceBuilder::build(run_id, &events, 0)))
    }
}
