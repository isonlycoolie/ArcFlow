# Docker Compose stacks

ArcFlow ships Compose files for local development, integration testing, and self-hosted production. Run commands from the repository root.

## Compose files

| File | Purpose | Typical audience |
|------|---------|------------------|
| [`docker-compose.dev.yml`](docker-compose.dev.yml) | Postgres and Qdrant for SDK and local engine work | Contributors |
| [`docker-compose.server.yml`](docker-compose.server.yml) | `arcflow-server` with Postgres (HTTP API integrators) | Backend developers |
| [`docker-compose.prod.yml`](docker-compose.prod.yml) | Production-like stack: migrate job, server, relay, Postgres, Qdrant | Self-hosters |
| [`docker-compose.otel.yml`](docker-compose.otel.yml) | Optional OpenTelemetry collector overlay (alpha) | Operators |

## Supporting config

| File | Purpose |
|------|---------|
| [`otel-collector-config.yaml`](otel-collector-config.yaml) | Collector config used with `docker-compose.otel.yml` |

## Deployment notes

| File | Notes |
|------|-------|
| [`observability-otel.md`](observability-otel.md) | OTel overlay usage |
| [`edge-deployment-cloudflare.md`](edge-deployment-cloudflare.md) | Edge / Cloudflare deployment patterns |

For full deployment guidance, see [documentation/deployment/overview.md](../documentation/deployment/overview.md).

## Quick start (local dependencies)

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

Production-like stack:

```bash
docker compose -f docker/docker-compose.prod.yml up --build
```

Set `ARCFLOW_SERVER_API_KEY` and `ARCFLOW_ADMIN_API_KEY` in your shell or a `.env` file before starting the prod stack. See [environment variables reference](../documentation/deployment/environment-variables-reference.md).
