**Audience:** `[operator]`

# Admin API reference

Complete reference for ArcFlow admin routes under `/v1/admin/*`. These routes implement the static product operator workflows defined in OSS `dashboard/spec/03-admin-api-contract.md`.

**Authentication:** all routes require:

```http
Authorization: Bearer <ARCFLOW_ADMIN_API_KEY>
```

Scoped runtime keys are rejected on admin routes (**403**). Admin routes return **401** when the key is missing or invalid.

Base URL: your server origin (e.g. `https://api.example.com` or `http://localhost:8080`).

Normative HTTP index (partially stale): [contracts/normative/runtime/server-api-v1.md](../../contracts/normative/runtime/server-api-v1.md). Authoritative admin shapes: `dashboard/spec/03-admin-api-contract.md`.

## POST /v1/admin/sites

Create a Relay site.

**Request:**

```json
{
  "display_name": "Acme Support",
  "allowed_origins": ["https://www.acme.com"],
  "rate_limit_rpm": 60,
  "allow_inline": false,
  "default_workflow_name": "chat",
  "upstream_runtime_key": null,
  "chat_instructions": null
}
```

| Field | Required | Notes |
|-------|----------|-------|
| `display_name` | Yes | Non-empty after trim; 3-80 chars recommended |
| `allowed_origins` | No | Default `[]`; production needs HTTPS origins |
| `rate_limit_rpm` | No | Default `60`, minimum `1` |
| `allow_inline` | No | Default `false` |
| `default_workflow_name` | No | Default `"chat"` |
| `upstream_runtime_key` | No | Falls back to `ARCFLOW_DEFAULT_UPSTREAM_RUNTIME_KEY` |
| `chat_instructions` | No | Optional seed for publish |

**Response 200:**

```json
{
  "site_id": "s_abc123",
  "relay_url": "http://localhost:8090/v1/sites/s_abc123",
  "site_token": "st_live_xxxxxxxx",
  "kb_namespace": "site_s_abc123"
}
```

`site_token` is returned **once**. Store it immediately; GET never returns the token.

**Errors:** 400 validation, 401 auth, 403 scoped key, 500 storage failure.

## GET /v1/admin/sites/{site_id}

Fetch site metadata.

**Response 200:**

```json
{
  "site_id": "s_abc123",
  "display_name": "Acme Support",
  "allowed_origins": ["https://www.acme.com"],
  "rate_limit_rpm": 60,
  "allow_inline": false,
  "default_workflow_name": "chat",
  "kb_namespace": "site_s_abc123",
  "chat_instructions": "You are Acme support.",
  "created_at": "2026-05-31T12:00:00Z"
}
```

**Errors:** 401, 403, 404 unknown site.

## PATCH /v1/admin/sites/{site_id}

Partial update. All body fields optional.

**Request example:**

```json
{
  "display_name": "Acme Support EU",
  "allowed_origins": ["https://eu.acme.com"],
  "rate_limit_rpm": 120,
  "allow_inline": false,
  "chat_instructions": "You are Acme EU support."
}
```

**Response 200:** same shape as GET.

**Errors:** 400 invalid origin or rate limit, 401, 403, 404.

## POST /v1/admin/sites/{site_id}/tokens/rotate

Invalidate previous site token; issue new token.

**Response 200:**

```json
{
  "site_token": "st_live_newtoken"
}
```

Shown once. Update frontend env and redeploy or reload config.

**Errors:** 401, 403, 404.

## POST /v1/admin/sites/{site_id}/knowledge/ingest

Ingest text into the site vector namespace (requires Postgres + Qdrant + embedding provider).

**Request:**

```json
{
  "text": "FAQ: How do I reset my password? ...",
  "key": "faq-password"
}
```

| Field | Required | Notes |
|-------|----------|-------|
| `text` | Yes | Non-empty; chunked server-side |
| `key` | No | Stable id for re-ingest overwrite |

**Response 200:**

```json
{
  "chunks_ingested": 12,
  "namespace": "site_s_abc123"
}
```

**Errors:** 400 empty text, 401, 403, 404, 503 Postgres/Qdrant unavailable.

## POST /v1/admin/sites/{site_id}/workflows/chat/publish

Publish default chat workflow to semver registry and bind site.

**Request:**

```json
{
  "instructions": "You are Acme Corp support. Answer from knowledge only.",
  "version": "1.0.0"
}
```

**Response 200:**

```json
{
  "name": "chat",
  "version": "1.0.0",
  "schema_hash": "blake3:..."
}
```

Browser clients call `runPublished("chat", "^1.0.0", message)` via static SDK.

**Errors:** 400 invalid semver, 401, 403, 404, 500 publish failure.

## Error shape summary

| Status | Meaning |
|--------|---------|
| 400 | Validation (empty name, bad origin, empty ingest text) |
| 401 | Missing or invalid admin key |
| 403 | Scoped runtime key attempted admin route |
| 404 | Unknown `site_id` |
| 500 | Server or storage failure |
| 503 | Postgres required but unavailable |

Dashboard UI (FP-3.01 deferred) maps 401 to key configuration screens. Until the private [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git) ships, use curl, HTTP clients, or OSS scripts:

- `scripts/static-provision-site.sh`
- `scripts/static-ingest-knowledge.sh`
