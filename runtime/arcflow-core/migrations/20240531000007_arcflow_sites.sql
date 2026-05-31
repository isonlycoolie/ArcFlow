-- Site registry for ArcFlow Relay.

CREATE TABLE IF NOT EXISTS arcflow_sites (
    id                  TEXT PRIMARY KEY,
    display_name        TEXT NOT NULL,
    allowed_origins     TEXT[] NOT NULL DEFAULT '{}',
    rate_limit_rpm      INTEGER NOT NULL DEFAULT 60,
    allow_inline        BOOLEAN NOT NULL DEFAULT FALSE,
    default_workflow_name TEXT,
    kb_namespace        TEXT NOT NULL,
    upstream_runtime_key TEXT NOT NULL,
    chat_instructions   TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS arcflow_site_tokens (
    site_id     TEXT NOT NULL REFERENCES arcflow_sites(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL,
    label       TEXT NOT NULL DEFAULT 'primary',
    revoked_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (site_id, token_hash)
);

CREATE INDEX IF NOT EXISTS idx_site_tokens_active
    ON arcflow_site_tokens (site_id) WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS arcflow_site_usage_daily (
    site_id     TEXT NOT NULL REFERENCES arcflow_sites(id) ON DELETE CASCADE,
    date        DATE NOT NULL,
    run_count   BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (site_id, date)
);
