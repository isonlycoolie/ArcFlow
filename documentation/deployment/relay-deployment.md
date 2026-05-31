**Audience:** `[operator]` `[platform]`

# Relay deployment

`arcflow-relay` is the browser-facing HTTP proxy for static product chat. It validates site tokens, enforces allowed origins, applies per-site rate limits, and forwards run requests to `arcflow-server` with a scoped upstream runtime key. Relay does not execute workflows; it is a security and policy boundary.

## Role in the architecture

```text
Browser (static SDK, mode: relay)
    → Relay (site token + Origin)
        → arcflow-server (scoped runtime key)
            → Postgres / Qdrant / LLM providers
```

LLM provider keys remain on the server. The browser holds only the site token and public Relay URL.

## Relay routes

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/v1/sites/{site_id}/runs` | Create run (proxied) |
| GET | `/v1/sites/{site_id}/runs/{run_id}` | Poll status |
| GET | `/v1/sites/{site_id}/runs/{run_id}/trace` | Metadata trace (SEC-1) |

Admin routes (`/v1/admin/*`) are never exposed through Relay.

## Environment variables

| Variable | Required | Purpose |
|----------|----------|---------|
| `ARCFLOW_UPSTREAM_URL` | Yes | Server base URL (e.g. `http://arcflow-server:8080`) |
| `ARCFLOW_RELAY_PORT` | No | Listen port (default 8090) |
| `ARCFLOW_POSTGRESQL_URL` | Recommended | Load sites from Postgres (admin-created sites) |
| `ARCFLOW_RELAY_SITES_JSON` | BYO alternative | Inline site config for Docker-only Relay |

When both Postgres and JSON are configured, Postgres-backed sites from admin API are authoritative for dynamic provisioning. BYO pattern: `examples/relay/byo-docker/`.

Optional admin response helper on server:

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_RELAY_PUBLIC_URL` | Public Relay URL returned in `POST /v1/admin/sites` responses |

## Stateless vs Postgres-backed

Relay process itself is stateless. Rate limit counters and site token hashes may be stored in Postgres when `ARCFLOW_POSTGRESQL_URL` is set (migrations 000007). No separate Relay database is required beyond sharing the server Postgres instance.

## Deploy with Compose

Production stack includes Relay:

```bash
docker compose -f docker/docker-compose.prod.yml up -d arcflow-relay
```

Verify upstream connectivity:

```bash
curl -sf http://localhost:8090/health || curl -sf http://localhost:8090/
```

(Exact health route depends on relay build; confirm upstream server `/ready` first.)

## Site provisioning flow

1. Operator calls `POST /v1/admin/sites` on the server (admin key).
2. Response includes `relay_url`, `site_id`, one-time `site_token`.
3. Frontend env: `VITE_ARCFLOW_RELAY_URL`, `VITE_ARCFLOW_SITE_ID`, `VITE_ARCFLOW_SITE_TOKEN`.
4. Static SDK calls Relay with `mode: "relay"`.

See [Sites management](../operator/sites-management.md).

## Middleware behavior

| Control | Behavior on failure |
|---------|---------------------|
| Origin check | Request rejected if `Origin`/`Referer` not in `allowed_origins` |
| Site token | 401 if missing or invalid Bearer token |
| Rate limit | 429 when `rate_limit_rpm` exceeded for site |
| Workflow allowlist | Scoped upstream key limits callable workflows |

Monitor 429 rates and origin validation failures in Relay logs. Spikes may indicate token leakage, missing origin configuration, or abuse.

## Security notes

- Site tokens are public to the frontend bundle by design; abuse is bounded by origin allowlist and RPM.
- Never use `mode: "direct"` in production (embeds server API key in browser). See [Relay security model](../security/relay-security-model.md).
- Rotate tokens via `POST /v1/admin/sites/{id}/tokens/rotate` on suspected exposure.

## Scaling

Run multiple Relay replicas behind a load balancer. All instances need the same upstream URL and Postgres URL. Rate limiting is per-site in-process; for strict global limits at scale, add edge rate limiting (CDN, API gateway).

## Related pages
