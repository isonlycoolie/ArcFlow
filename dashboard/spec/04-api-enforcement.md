# API enforcement

## Admin authentication

All `/v1/admin/*` routes require:

```http
Authorization: Bearer <ARCFLOW_ADMIN_API_KEY>
```

The server validates via `require_admin_key` middleware. Missing or wrong key → **401 Unauthorized**.

## Runtime API key (non-admin)

Runtime routes (`/v1/runs`, `/v1/workflows`, etc.) accept:

```http
Authorization: Bearer <ARCFLOW_API_KEY>
```

Keys may be scoped. Admin routes **reject** scoped keys that lack admin scope → **403 Forbidden**.

## Enforcement matrix

| Route pattern | Admin key | Full runtime key | Scoped site key |
|---------------|-----------|------------------|-----------------|
| `POST /v1/admin/sites` | Yes | No | No |
| `GET/PATCH /v1/admin/sites/*` | Yes | No | No |
| `POST .../tokens/rotate` | Yes | No | No |
| `POST .../knowledge/ingest` | Yes | No | No |
| `POST .../workflows/chat/publish` | Yes | No | No |
| `POST /v1/runs` | No | Yes | Per policy |
| Relay `POST /v1/sites/{id}/runs` | No | No | Site token only |

## Dashboard BFF pattern (recommended)

The browser must **not** hold `ARCFLOW_ADMIN_API_KEY`. Production deployments use a small BFF (Next.js API routes, Cloudflare Worker, etc.) that:

1. Authenticates the operator session (cookie, SSO, or static operator password)
2. Injects `Authorization: Bearer` server-side when calling ArcFlow admin API
3. Never forwards admin key to client bundles

Direct browser → admin API is acceptable only for local development with a dev-only key.

## Rate limits

Admin routes inherit server body limit (1 MiB). Relay enforces per-site `rate_limit_rpm` on public chat runs, not on admin calls.

## Validation rules (server-enforced)

| Input | Rule |
|-------|------|
| `display_name` | Non-empty after trim |
| `allowed_origins` | Each origin must be valid URL; production sites should use HTTPS |
| `rate_limit_rpm` | ≥ 1 |
| Ingest `text` | Non-empty |
| Publish `version` | Valid semver when provided |

Dashboard should mirror these rules client-side for fast feedback.
