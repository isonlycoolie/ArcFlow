
# BYO Relay deployment

Bring Your Own (BYO) Relay runs `arcflow-relay` on your infrastructure with the same browser contract as the Compose-managed Relay in `docker/docker-compose.server.yml`. Use BYO when you need a custom domain, separate network zone, or air-gapped edge while keeping `arcflow-server` centralized.

Reference implementation: `examples/relay/byo-docker/`.

## Quick start

```bash
export ARCFLOW_UPSTREAM_URL=http://localhost:8080
export ARCFLOW_RELAY_SITES_JSON='[{
 "id": "s_dev",
 "display_name": "Dev",
 "allowed_origins": ["http://localhost:5173"],
 "rate_limit_rpm": 60,
 "allow_inline": false,
 "default_workflow_name": "chat",
 "kb_namespace": "site-s_dev-kb",
 "upstream_runtime_key": "dev-secret",
 "token": "st_live_devtoken"
}]'
docker compose -f examples/relay/byo-docker/compose.yml up --build
```

Browser env:

```bash
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/s_dev
VITE_ARCFLOW_SITE_TOKEN=st_live_devtoken
```

Health check:

```bash
curl -sf http://localhost:8090/health
```

## ARCFLOW_RELAY_SITES_JSON schema

Each array element maps to a site record:

| Field | Required | Description |
|-------|:--------:|-------------|
| `id` | Yes | Site id in URL path |
| `display_name` | Yes | Operator label |
| `allowed_origins` | Yes | Origin allowlist |
| `rate_limit_rpm` | Yes | Per-minute cap |
| `allow_inline` | Yes | Browser inline workflow override |
| `default_workflow_name` | No | Default publish name |
| `kb_namespace` | Yes | Vector namespace |
| `upstream_runtime_key` | Yes | Key sent to server (must match scoped key or master) |
| `token` | Yes | Plain site token Relay accepts from browsers |


## Environment variables

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_UPSTREAM_URL` | Server base URL (e.g. `https://api.internal:8080`) |
| `ARCFLOW_RELAY_SITES_JSON` | Inline site definitions (BYO) |
| `ARCFLOW_RELAY_PORT` | Listen port (default 8090) |
| `ARCFLOW_RELAY_PUBLIC_URL` | Returned in admin site create when server provisions sites |

Server-side companion vars:

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_STATIC_RUNTIME_KEYS` | JSON map limiting upstream keys per workflow |
| `ARCFLOW_SERVER_API_KEY` | Master key if upstream uses it |
| `ARCFLOW_RELAY_PUBLIC_URL` | Public Relay base for admin responses |

Full list: [Environment variables reference](../deployment/environment-variables-reference.md).

## Admin-provisioned vs BYO

| Approach | Site source | Best for |
|----------|-------------|----------|
| Admin API | Postgres `arcflow_sites` (future Relay sync) | Operator dashboard, dynamic sites |
| BYO JSON | Env at Relay start | Fixed sites, edge deploy, CI smoke |

For production with changing sites, prefer admin API provisioning and plan Relay config reload strategy. BYO JSON requires redeploy to add sites.

## Scoped upstream keys

Define scoped keys on the server:

```json
{
 "relay-site-key-abc": {
 "workflows": ["chat"],
 "publish": false
 }
}
```

Set `"upstream_runtime_key": "relay-site-key-abc"` in site JSON. Never put the master server key in Relay env if a scoped key suffices.

## TLS and networking

Typical layout:

```text
Internet → CDN/WAF → Relay (TLS termination)
 → private network → arcflow-server
```

Relay does not terminate LLM traffic; it only proxies JSON run APIs.

## Smoke test

After deploy, run repository scripts if available:

```bash
./scripts/static-smoke.sh
```

Or manual curl from [Origin and rate limiting](origin-and-rate-limiting.md).

## Related pages

- [Overview](overview.md)
- [Request path](request-path.md)
- [Relay BYO deployment](../examples/relay-byo-deployment.md) (when published)
