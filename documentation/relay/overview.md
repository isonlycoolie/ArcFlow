
# ArcFlow Relay overview

`arcflow-relay` is a stateless HTTP proxy between browser clients and `arcflow-server`. It exists so production static sites can run published workflows without placing LLM provider keys or the master server API key in frontend bundles.

Relay holds **site tokens** (scoped, rotatable) instead of OpenAI/Anthropic keys. Origin validation and per-site rate limits reduce abuse if a token leaks.

## Why Relay exists

| Without Relay | With Relay |
|---------------|------------|
| Browser needs server API key | Browser sends site token only |
| CORS wide open on server | Origin checked per site |
| No per-site rate limit | `rate_limit_rpm` enforced |
| Full registry access risk | Upstream uses scoped runtime key |

Relay is Postgres-free. Site configuration comes from admin-provisioned records (server Postgres) or `ARCFLOW_RELAY_SITES_JSON` for BYO deployments.

## Three-party request path

```text
Browser (static SDK, site token)
 → arcflow-relay :8090
 → Origin check
 → Rate limit check
 → POST /v1/runs (scoped upstream key)
 → arcflow-server :8080
 → arcflow-core execution
 ← run_id, status
```

Detail: [Request path](request-path.md).

## Relay routes

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness |
| POST | `/v1/sites/{site_id}/runs` | Create run (proxied) |
| GET | `/v1/sites/{site_id}/runs/{run_id}` | Poll status |
| GET | `/v1/sites/{site_id}/runs/{run_id}/trace` | Metadata trace |

Auth: `Authorization: Bearer <site_token>` plus allowed `Origin` header on mutating requests.

## Server SSE not available (streaming deferred)

Relay does not expose SSE. Browser streaming UX uses trace polling for `TokenEmitted` metadata events. Server-side `GET /v1/runs/{id}/events` is deferred (streaming deferred).

## Deployment options

| Mode | When |
|------|------|
| Docker Compose (`docker-compose.server.yml`) | Local dev with server stack |
| Admin-provisioned `relay_url` | Operator creates site via admin API |
| BYO Relay | Self-hosted binary with JSON site config |

BYO guide: [BYO Relay deployment](byo-relay-deployment.md).

## Related pages

- [Origin and rate limiting](origin-and-rate-limiting.md)
- [Static product overview](../static-product/overview.md)
- [Security model](../static-product/security-model.md)
