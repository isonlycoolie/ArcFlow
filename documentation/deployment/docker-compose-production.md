**Audience:** `[platform]`

# Docker Compose production stack

`docker/docker-compose.prod.yml` models a production-like deployment: Postgres and Qdrant with named volumes, a one-shot migration job, `arcflow-server`, and `arcflow-relay`. Use it for staging, single-node production, or as a reference when translating to Kubernetes manifests.

## Services

| Service | Role | Notes |
|---------|------|-------|
| `postgres` | Primary datastore | Volume `arcflow_pg_data` |
| `qdrant` | Vector store | Volume `arcflow_qdrant_data`; port 6333 exposed |
| `arcflow-migrate` | Schema apply | `restart: "no"`; must complete before server |
| `arcflow-server` | Runtime HTTP API | Port 8080; non-root in Dockerfile |
| `arcflow-relay` | Browser proxy | Port 8090; upstream to server |

Dependency order: Postgres healthy → migrate success → server start → relay start.

## Start

Set secrets in the shell or a local `.env` file (never commit):

```bash
export ARCFLOW_SERVER_API_KEY="$(openssl rand -hex 32)"
export ARCFLOW_ADMIN_API_KEY="$(openssl rand -hex 32)"
export ARCFLOW_EMBEDDING_PROVIDER=openai/text-embedding-3-small
export OPENAI_API_KEY=sk-your-production-key

docker compose -f docker/docker-compose.prod.yml up --build -d
```

Wait for migrate completion, then verify:

```bash
curl -sf http://localhost:8080/ready | jq .
curl -sf http://localhost:8080/health | jq .
```

## Production differences from dev compose

| Aspect | Dev (`docker-compose.dev.yml`) | Prod (`docker-compose.prod.yml`) |
|--------|--------------------------------|----------------------------------|
| ArcFlow binaries | Host-built | Container-built |
| Data persistence | Ephemeral | Named volumes |
| Migration job | Manual or separate | Compose `arcflow-migrate` service |
| Qdrant | Exposed for local tools | Required for site knowledge |
| Relay | Optional | Included |
| API keys | Hardcoded dev defaults in server compose | Must inject via environment |

## Container hardening

Official server Dockerfile (`server/arcflow-server/Dockerfile`):

- Multi-stage build from `rust:1.77-slim-bookworm`
- Runtime image `debian:bookworm-slim`
- Dedicated user `arcflow` (uid 1000, gid 1000)
- `USER arcflow` before `ENTRYPOINT`
- Built-in `HEALTHCHECK` against `GET /health`

Relay Dockerfile follows the same non-root pattern. Do not run production containers as root.

## Resource and restart policy

Compose file defaults do not set CPU/memory limits. For production hosts, add:

```yaml
deploy:
  resources:
    limits:
      cpus: "2"
      memory: 2G
  restart_policy:
    condition: on-failure
```

Or use orchestrator-level limits in Kubernetes.

Recommended restart policies:

| Service | Policy |
|---------|--------|
| postgres, qdrant, server, relay | `unless-stopped` or orchestrator equivalent |
| arcflow-migrate | `no` (one-shot) |

## Observability overlay

Optional OpenTelemetry collector:

```bash
docker compose -f docker/docker-compose.prod.yml -f docker/docker-compose.otel.yml up -d
```

Set `ARCFLOW_OTEL_ENABLED=true` and `ARCFLOW_OTLP_ENDPOINT` on the server. OTel metrics remain alpha under FP-4; core operation does not require them.

See `docker/observability-otel.md` and [OpenTelemetry guide](../guides/observability/opentelemetry.md).

## When to prefer Kubernetes over Compose

Compose suits single-node or small teams. Move to Kubernetes when you need:

- Horizontal pod autoscaling for server or relay
- Managed Postgres/Qdrant instead of container volumes
- Secret stores (Vault, cloud SM) integrated at pod level
- Zero-downtime rolling deploys with readiness probes

Translate the same startup order: migrate job as init container or Helm pre-hook, then server Deployment with `/ready` probe.

## HTTPS and network boundary

Compose exposes plain HTTP on 8080 and 8090. Terminate TLS at a reverse proxy or cloud load balancer. Restrict Postgres and Qdrant ports to internal networks only in production.

## Related pages

- [Server deployment](server-deployment.md)
- [Relay deployment](relay-deployment.md)
- [Production checklist](production-checklist.md)
- [Self-hosted security](../security/self-hosted-security.md)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §23.1; `docker/docker-compose.prod.yml`, `server/arcflow-server/Dockerfile`.
