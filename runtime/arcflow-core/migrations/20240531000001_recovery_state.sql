-- ArcFlow recovery state (Sprint 7).

CREATE TABLE IF NOT EXISTS arcflow_recovery_state (
    recovery_id         TEXT PRIMARY KEY,
    original_run_id     TEXT NOT NULL UNIQUE,
    workflow_def_id     TEXT NOT NULL,
    state_json          JSONB NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_consumed         BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT valid_run_id CHECK (original_run_id <> '')
);

CREATE INDEX IF NOT EXISTS idx_recovery_original_run
    ON arcflow_recovery_state (original_run_id);

CREATE INDEX IF NOT EXISTS idx_recovery_workflow_unconsumed
    ON arcflow_recovery_state (workflow_def_id, is_consumed);
