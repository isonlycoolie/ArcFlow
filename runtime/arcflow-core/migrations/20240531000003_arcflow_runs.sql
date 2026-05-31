-- arcflow_runs table for HTTP server run tracking.

CREATE TABLE IF NOT EXISTS arcflow_runs (
    run_id           TEXT PRIMARY KEY,
    trace_id         TEXT NOT NULL,
    status           TEXT NOT NULL,
    workflow_hash    TEXT NOT NULL,
    exec_config_json JSONB,
    result_json      JSONB,
    error_json       JSONB,
    idempotency_key  TEXT UNIQUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at       TIMESTAMPTZ,
    completed_at     TIMESTAMPTZ,
    CONSTRAINT valid_run_id CHECK (run_id <> '')
);

CREATE INDEX IF NOT EXISTS idx_runs_status ON arcflow_runs (status);
CREATE INDEX IF NOT EXISTS idx_runs_created ON arcflow_runs (created_at DESC);
