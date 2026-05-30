-- Migration 006: versioned workflow registry (Phase 3.3).

CREATE TABLE IF NOT EXISTS arcflow_workflows (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    version         TEXT NOT NULL,
    schema_hash     TEXT NOT NULL,
    definition_json JSONB NOT NULL,
    published_by    TEXT,
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deprecated      BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (name, version)
);

CREATE INDEX IF NOT EXISTS idx_workflows_name ON arcflow_workflows (name, published_at DESC);

CREATE TABLE IF NOT EXISTS arcflow_workflow_aliases (
    name            TEXT NOT NULL,
    alias           TEXT NOT NULL,
    version         TEXT NOT NULL,
    PRIMARY KEY (name, alias)
);
