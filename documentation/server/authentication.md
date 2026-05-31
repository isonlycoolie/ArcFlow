**Audience:** `[platform]` `[compliance]`

# Server authentication

Every `arcflow-server` route belongs to an auth tier. Requests that fail authentication receive **401** with a JSON error body. Admin and runtime keys use constant-time comparison; never log key values.

## Route groups

| Route group | Auth | Example paths |
|-------------|------|---------------|
| Public | None | `GET /health`, `GET /ready` |
| Runtime | `ARCFLOW_SERVER_API_KEY` | `/v1/runs`, `/v1/workflows/*`, trace, approve, external |
| Admin | `ARCFLOW_ADMIN_API_KEY` | `/v1/admin/sites`, knowledge ingest, chat publish |
| Debug | Localhost + `ARCFLOW_DEBUG=true` | `/v1/debug/*` (feature-gated build) |
| Relay (separate binary) | Site token + Origin allowlist | `/v1/sites/{site_id}/runs` |

Relay auth is documented in [relay/overview.md](../relay/overview.md). This page covers `arcflow-server` only.

## Runtime API key

Accepted headers (either works):

- `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>`
- `X-ArcFlow-Api-Key: <ARCFLOW_SERVER_API_KEY>`

Example:

```bash
curl -s http://localhost:8080/v1/runs/RUN_ID \
  -H "Authorization: Bearer dev-secret"
```

```bash
curl -s http://localhost:8080/v1/runs/RUN_ID \
  -H "X-ArcFlow-Api-Key: dev-secret"
```

Missing or wrong keys return **401**:

```json
{
  "error": {
    "code": "authentication_failed",
    "message": "[ArcFlow] Authentication failed. Provide X-ArcFlow-Api-Key or Authorization: Bearer."
  }
}
```

Set `ARCFLOW_SERVER_API_KEY` in the server environment. Local Docker Compose uses `dev-secret` for development only. Production keys should be long random strings stored in a secret manager, not in source control.

## Admin API key

Admin routes require:

```bash
curl -s -X POST http://localhost:8080/v1/admin/sites \
  -H "Authorization: Bearer dev-admin" \
  -H "Content-Type: application/json" \
  -d '{"display_name":"Acme","allowed_origins":["https://www.acme.com"],"rate_limit_rpm":60,"allow_inline":false}'
```

The runtime key must **not** work on `/v1/admin/*`. Separate keys limit blast radius if a frontend site token or backend integration key leaks.

## Scoped runtime keys (Relay upstream)

`ARCFLOW_STATIC_RUNTIME_KEYS` is a JSON map from key string to scope:

```json
{
  "relay-site-key-abc": {
    "workflows": ["chat"],
    "publish": false
  }
}
```

When Relay calls `POST /v1/runs` upstream, it uses the site's `upstream_runtime_key`. Scoped keys may only start runs for listed workflow names. Attempts outside scope return **403**.

Master key (`ARCFLOW_SERVER_API_KEY`) bypasses workflow name restrictions.

## Site tokens (Relay, not server)

Browsers send `Authorization: Bearer <site_token>` to Relay. Site tokens are provisioned once via `POST /v1/admin/sites` and rotated via `POST /v1/admin/sites/{id}/tokens/rotate`. They are scoped to a site, not equivalent to LLM provider keys.

## Debug endpoints

Debug routes are compiled behind the `debug-endpoints` feature and gated at runtime by localhost client IP and `ARCFLOW_DEBUG=true`. Do not enable in production-facing deployments.

## Key rotation

| Key | Rotation procedure |
|-----|-------------------|
| `ARCFLOW_SERVER_API_KEY` | Deploy new value, update all backend callers and Relay `upstream_runtime_key` entries, revoke old value |
| `ARCFLOW_ADMIN_API_KEY` | Deploy new value, update operator scripts and CI provision jobs |
| Site token | `POST /v1/admin/sites/{id}/tokens/rotate`, update frontend env (Vite/build injection) |
