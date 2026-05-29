-- ArcFlow recovery state v2 (Phase 1.1 graph checkpoints).
-- Apply after recovery-schema-v1.sql.

ALTER TABLE arcflow_recovery_state
    ADD COLUMN IF NOT EXISTS execution_mode TEXT NOT NULL DEFAULT 'linear';

ALTER TABLE arcflow_recovery_state
    ADD COLUMN IF NOT EXISTS current_node_id TEXT;

ALTER TABLE arcflow_recovery_state
    ADD COLUMN IF NOT EXISTS graph_iteration_count INTEGER NOT NULL DEFAULT 0;

-- Run tracking for HTTP server (Phase 1.3).
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

-- Human-in-the-loop approvals (Phase 1.4).
CREATE TABLE IF NOT EXISTS arcflow_human_approvals (
    run_id          TEXT NOT NULL,
    approval_key    TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    approved        BOOLEAN,
    data_json       JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at     TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (run_id, approval_key)
);

CREATE INDEX IF NOT EXISTS idx_approvals_pending
    ON arcflow_human_approvals (status, expires_at)
    WHERE status = 'pending';

ALTER TABLE arcflow_runs ADD COLUMN IF NOT EXISTS workflow_json JSONB;
ALTER TABLE arcflow_runs ADD COLUMN IF NOT EXISTS agents_json JSONB;
ALTER TABLE arcflow_runs ADD COLUMN IF NOT EXISTS input_text TEXT;
