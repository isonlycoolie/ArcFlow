# Relay BYO (Bring Your Own)

Self-hosted ArcFlow Relay with the same browser contract as managed Relay.

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
docker compose -f compose.yml up --build
```

Browser env:

```bash
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/s_dev
VITE_ARCFLOW_SITE_TOKEN=st_live_devtoken
```

## Routes

| Method | Path |
|--------|------|
| GET | `/health` |
| POST | `/v1/sites/{site_id}/runs` |
| GET | `/v1/sites/{site_id}/runs/{run_id}` |
| GET | `/v1/sites/{site_id}/runs/{run_id}/trace` |

Auth: `Authorization: Bearer {site_token}` + allowed `Origin` header.
