-- Migration 002: graph checkpoint columns on arcflow_recovery_state.
-- Idempotent for CI and local Docker Postgres.

ALTER TABLE arcflow_recovery_state
    ADD COLUMN IF NOT EXISTS execution_mode TEXT NOT NULL DEFAULT 'linear';

ALTER TABLE arcflow_recovery_state
    ADD COLUMN IF NOT EXISTS current_node_id TEXT;

ALTER TABLE arcflow_recovery_state
    ADD COLUMN IF NOT EXISTS graph_iteration_count INTEGER NOT NULL DEFAULT 0;
