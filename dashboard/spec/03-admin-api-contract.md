# Admin API contract

Base URL: `ARCFLOW_ADMIN_URL` (e.g. `http://localhost:8080`).

Authentication: see [04-api-enforcement.md](04-api-enforcement.md).

## POST /v1/admin/sites

Create a Relay site.

**Request**

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
| `display_name` | Yes | 3–80 chars recommended |
| `allowed_origins` | No | Default `[]`; production requires at least one HTTPS origin |
| `rate_limit_rpm` | No | Default `60`, minimum `1` |
| `allow_inline` | No | Default `false`; Tier 1 keeps inline workflows off |
| `default_workflow_name` | No | Default `"chat"` |
| `upstream_runtime_key` | No | Falls back to server default runtime key |
| `chat_instructions` | No | Optional seed; publish step can override |

**Response 200**

```json
{
  "site_id": "s_abc123",
  "relay_url": "http://localhost:8090/v1/sites/s_abc123",
  "site_token": "st_live_xxxxxxxx",
  "kb_namespace": "site_s_abc123"
}
```

`site_token` is shown **once**. Dashboard must warn the operator to save it.

## GET /v1/admin/sites/{site_id}

**Response 200**

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

Token is never returned on GET.

## PATCH /v1/admin/sites/{site_id}

**Request** (all fields optional)

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

## POST /v1/admin/sites/{site_id}/tokens/rotate

Invalidates the previous site token.

**Response 200**

```json
{
  "site_token": "st_live_newtoken"
}
```

## POST /v1/admin/sites/{site_id}/knowledge/ingest

**Request**

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

**Response 200**

```json
{
  "chunks_ingested": 12,
  "namespace": "site_s_abc123"
}
```

## POST /v1/admin/sites/{site_id}/workflows/chat/publish

Publishes the default chat workflow to the registry and binds the site.

**Request**

```json
{
  "instructions": "You are Acme Corp support. Answer from knowledge only.",
  "version": "1.0.0"
}
```

**Response 200**

```json
{
  "name": "chat",
  "version": "1.0.0",
  "schema_hash": "blake3:..."
}
```

## Error shape

Admin routes return plain text or JSON error strings with HTTP status:

| Status | Meaning |
|--------|---------|
| 400 | Validation (empty name, bad origin) |
| 401 | Missing or invalid admin key |
| 403 | Scoped runtime key attempted admin route |
| 404 | Unknown `site_id` |
| 500 | Server or storage failure |

Dashboard should map 401 to a session/key configuration screen.

## Future (not R1)

- `GET /v1/admin/sites` list endpoint
- Workflow registry browser with semver range picker
