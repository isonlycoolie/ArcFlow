
# Deployment overview

ArcFlow ships as Rust binaries (`arcflow-server`, `arcflow-relay`, `arcflow` CLI) plus Docker Compose stacks under `docker/`. Production deployments center on three scenarios: local development, self-hosted server with Postgres, and the static product pattern (server + Relay + browser SDK).

Each scenario maps to a Compose file or bare-metal process layout. The server is the control plane for runs, registry, admin sites, traces, and recovery. Relay is optional but required for browser-facing chat without exposing LLM keys. Postgres is required for server runs, admin APIs, and trace persistence. Qdrant is required when vector memory or site knowledge ingest uses embeddings.

## Deployment scenarios

| Scenario | Primary audience | Compose file | Minimum services |
|----------|------------------|--------------|------------------|
| Local development | Developer | `docker/docker-compose.dev.yml` | Postgres, Qdrant |
| Server API + admin | Platform | `docker/docker-compose.server.yml` | Postgres, migrate job, server |
| Production static product | Operator, platform | `docker/docker-compose.prod.yml` | Postgres, Qdrant, migrate, server, relay |
| BYO Relay only | Frontend team | `examples/relay/byo-docker/` | Relay + upstream server URL |

### Local development

Developers run Postgres and Qdrant via `docker-compose.dev.yml`, then build the SDK or server on the host. This stack does not start ArcFlow binaries; it provides dependencies for `cargo run`, SDK tests, and RAG examples.

### Self-hosted server

`docker-compose.server.yml` builds `arcflow-migrate` (one-shot), `arcflow-server`, and optionally `arcflow-relay`. Use this when operators call `/v1/runs` and admin routes directly from backend services or CI, without a public browser chat widget.

### Static product (Relay + published workflows)

`docker-compose.prod.yml` adds Qdrant and production-oriented volume mounts. Operators create sites via admin API, ingest knowledge, publish chat workflows, and embed Relay URL plus site token in a static frontend. See [Relay deployment](relay-deployment.md) and [Sites management](../operator/sites-management.md).

## Startup sequence (server)

Regardless of Compose or bare metal, a healthy server deployment follows this order:

1. Validate environment (`ARCFLOW_SERVER_API_KEY`, `ARCFLOW_POSTGRESQL_URL`, provider keys as needed).
2. Ensure Postgres is reachable.
3. Apply migrations (`arcflow migrate up` or rely on auto-migrate, then verify `/ready`).
4. Start `arcflow-server` (non-root user in official Dockerfile).
5. Confirm `GET /health` returns 200 and `GET /ready` returns 200 before routing traffic.

Relay starts after the upstream server is listening. Relay reads site configuration from Postgres when `ARCFLOW_POSTGRESQL_URL` is set, or from `ARCFLOW_RELAY_SITES_JSON` for BYO deployments.

## Docker Compose file reference

| File | Purpose |
|------|---------|
| `docker/docker-compose.dev.yml` | Postgres 16 + Qdrant for host-side dev |
| `docker/docker-compose.server.yml` | Migrate + server + relay + static demo nginx |
| `docker/docker-compose.prod.yml` | Production-like stack with named volumes |
| `docker/docker-compose.otel.yml` | Optional OTel collector overlay (FP-4 alpha) |

Contract guides: [contracts/guides/deployment/self-hosted.md](../../contracts/guides/deployment/self-hosted.md), [meta-repo.md](../../contracts/guides/deployment/meta-repo.md).

Normative HTTP contract (stale in places): [HTTP API reference](../server/http-api-reference.md). Prefer [HTTP API reference](../server/http-api-reference.md) or [Admin API reference](../operator/admin-api-reference.md) for current routes.

## Operator dashboard

The operator UI lives in the private [ArcFlow-WebApp](https://github.com/isonlycoolie/ArcFlow-WebApp.git) repository (`webapp/` submodule). Tier 1 flows (sites, knowledge, chat publish, keys) are implemented. Tier 2 usage charts and server-side site list remain future work. See [Dashboard spec](../operator/dashboard-spec.md).

## Meta-repo layout

Private platform repos submodule OSS ArcFlow and ArcFlow-WebApp. Template: `deploy/meta-repo-template/`. Convention ports: server 8080, relay 8090, webapp dev 5174, operator-api 8091.

## Related pages

- [Docker Compose local](docker-compose-local.md)
- [Docker Compose production](docker-compose-production.md)
- [Server deployment](server-deployment.md)
- [Relay deployment](relay-deployment.md)
- [Migrations runbook](migrations-runbook.md)
- [Environment variables](environment-variables-reference.md)
- [Health and readiness](health-and-readiness.md)
- [Production checklist](production-checklist.md)
