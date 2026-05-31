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
