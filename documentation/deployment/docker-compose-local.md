
# Docker Compose local development stack

The local development stack provides Postgres and Qdrant on fixed host ports so SDK, server, and example workflows can run outside Docker while sharing consistent dependency versions.

File: `docker/docker-compose.dev.yml`.

## Services

| Service | Image | Host ports | Purpose |
|---------|-------|------------|---------|
| `postgres` | `postgres:16-alpine` | 5432 | Runs, recovery, registry, sites, traces |
| `qdrant` | `qdrant/qdrant:v1.12.5` | 6333, 6334 | Vector memory and RAG examples |

Both services define Docker healthchecks (`pg_isready` for Postgres, Qdrant `/readyz` for Qdrant).

## Start and verify

From the repository root:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

Verify Postgres:

```bash
docker compose -f docker/docker-compose.dev.yml exec postgres pg_isready -U arcflow -d arcflow
```

Verify Qdrant:

```bash
curl -sf http://localhost:6333/readyz
```

Default credentials (development only):

| Setting | Value |
|---------|-------|
| User | `arcflow` |
| Password | `arcflow` |
| Database | `arcflow` |
| Connection URL | `postgres://arcflow:arcflow@localhost:5432/arcflow` |

## Local `.env` template

Create a `.env` in the repo root or export variables in your shell. Never commit secrets.

```bash
# Server (when running arcflow-server on host)
export ARCFLOW_SERVER_API_KEY=dev-secret-change-me
export ARCFLOW_ADMIN_API_KEY=dev-admin-change-me
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow
export ARCFLOW_QDRANT_URL=http://localhost:6333
export ARCFLOW_EMBEDDING_PROVIDER=openai/text-embedding-3-small
export OPENAI_API_KEY=sk-your-key

# Optional hybrid retrieval
export ARCFLOW_QDRANT_HYBRID=true
export COHERE_API_KEY=your-cohere-key
```

Use `stub` embedding provider only for offline tests without real embeddings.

## Feature to service mapping

| Feature | Requires |
|---------|----------|
| SDK linear workflows (in-process) | LLM key or stub provider only |
| Server `POST /v1/runs` | Postgres + migrations |
| Recovery / HITL | Postgres + migrations |
| Vector RAG / knowledge ingest | Qdrant + real embedding provider |
| Hybrid retrieval + rerank | Qdrant + `ARCFLOW_QDRANT_HYBRID` + Cohere key |
| Admin sites / static product | Postgres + server + relay (see prod compose) |

## Running the server against this stack

After the dev stack is healthy:

```bash
arcflow migrate up
cargo run -p arcflow-server
```

Smoke checks:

```bash
curl -s http://localhost:8080/health
curl -s http://localhost:8080/ready
```

## Smoke test scripts

Reference implementations:

```bash
bash scripts/load-test-runs.sh
bash scripts/static-smoke.sh # requires full server + relay + site provisioned
```

## Stop and reset

```bash
docker compose -f docker/docker-compose.dev.yml down
```

To wipe data volumes (destructive):

```bash
docker compose -f docker/docker-compose.dev.yml down -v
```

## Related pages

- [Deployment overview](overview.md)
- [Environment variables reference](environment-variables-reference.md)
- [Install and build](../getting-started/install-and-build.md)
