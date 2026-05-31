**Audience:** `[operator]` `[compliance]`

# Token rotation

Procedure for rotating site tokens and server API keys without unnecessary downtime. Site tokens are scoped credentials embedded in static frontends. Server and admin keys protect backend APIs.

## When to rotate

| Trigger | Rotate |
|---------|--------|
| Suspected token exposure (git leak, support bundle) | Site token immediately |
| Scheduled security policy (e.g. quarterly) | Site token on schedule |
| Team member with access to site env leaves | Site token + review admin key |
| Admin key stored on decommissioned BFF | `ARCFLOW_ADMIN_API_KEY` |
| Server key in old CI logs | `ARCFLOW_SERVER_API_KEY` (coordinate with Relay upstream keys) |

## Site token rotation

### Step 1: Rotate via admin API

```bash
curl -sf -X POST "$ARCFLOW_ADMIN_URL/v1/admin/sites/s_abc123/tokens/rotate" \
  -H "Authorization: Bearer $ARCFLOW_ADMIN_API_KEY"
```

Response:

```json
{
  "site_token": "st_live_newtoken"
}
```

The previous token is invalidated immediately at Relay.

### Step 2: Dual-token window (optional zero-downtime)

ArcFlow does not support two active site tokens per site in R1. For zero-downtime static deploys:

1. Build new frontend bundle with new token.
2. Deploy to CDN/hosting.
3. Rotate token only after new bundle is live **or** accept brief chat outage during single-step rotation.

Alternative: deploy behind feature flag, rotate, then flip traffic.

### Step 3: Update frontend configuration

Update build-time env:

```bash
VITE_ARCFLOW_SITE_TOKEN=st_live_newtoken
```

Redeploy static assets. Do not commit tokens to git.

### Step 4: Verify

```bash
# Old token should fail
curl -X POST "https://relay.example.com/v1/sites/s_abc123/runs" \
  -H "Authorization: Bearer st_live_old" \
  -H "Origin: https://www.acme.com" \
  -H "Content-Type: application/json" \
  -d '{"input":"test"}'

# New token should succeed (with valid publish + workflow)
```

Run `bash scripts/static-smoke.sh` after rotation.

## ARCFLOW_SERVER_API_KEY rotation

Server key rotation affects all runtime clients (backend services, Relay upstream scoped keys, CI).

1. Generate new key: `openssl rand -hex 32`.
2. Update secret store and rolling deploy server replicas with new key.
3. Update `ARCFLOW_STATIC_RUNTIME_KEYS` and `ARCFLOW_DEFAULT_UPSTREAM_RUNTIME_KEY` if key ids changed.
4. Update Relay upstream configuration if it uses the raw server key.
5. Revoke old key from secret store after all clients updated.

Coordinate with platform team; mistimed rotation causes 401 on `/v1/runs`.

## ARCFLOW_ADMIN_API_KEY rotation

1. Generate new admin key.
2. Update operator BFF or CI secrets (never browser).
3. Rolling restart server with new `ARCFLOW_ADMIN_API_KEY`.
4. Invalidate old key.

Dashboard dev (private repo): update local `.env` on port 5174 only for development.

## Compliance notes

- Log rotation events in your change management system (who, when, site id).
- Do not log token values in application or support logs.
