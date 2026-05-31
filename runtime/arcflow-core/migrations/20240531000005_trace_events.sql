-- Persisted trace events for GET /v1/runs/{id}/trace.

CREATE TABLE IF NOT EXISTS arcflow_trace_events (
    run_id      TEXT NOT NULL,
    seq         BIGINT NOT NULL,
    event_json  JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (run_id, seq),
    CONSTRAINT valid_trace_run_id CHECK (run_id <> '')
);

CREATE INDEX IF NOT EXISTS idx_trace_events_run ON arcflow_trace_events (run_id);
