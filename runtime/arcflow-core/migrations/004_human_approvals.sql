-- Migration 004: human approvals and run payload snapshots (Phase 1.4 HITL).

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
