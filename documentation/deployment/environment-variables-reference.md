
# Environment variables reference

Complete reference for ArcFlow environment variables across server, Relay, CLI, and tooling. Normative summary: [Environment variables reference](../deployment/environment-variables-reference.md). Never commit `.env` files containing secrets to source control.

## Server authentication

| Variable | Required | Default | Example | Surface | Log classification |
|----------|----------|---------|---------|---------|-------------------|
| `ARCFLOW_SERVER_API_KEY` | Yes (server) | none | `a1b2...` (64 hex) | Server runtime routes | **Never log** |
| `ARCFLOW_ADMIN_API_KEY` | Yes for admin | none | separate 64 hex | `/v1/admin/*` | **Never log** |
| `ARCFLOW_STATIC_RUNTIME_KEYS` | No | none | JSON map | Scoped workflow keys | **Never log** |
| `ARCFLOW_DEFAULT_UPSTREAM_RUNTIME_KEY` | For admin sites | none | matches scoped key id | Site create upstream | **Never log** |

Static keys JSON example:

```json
{
  "site-key-1": {
    "workflows": ["chat"],
    "publish": false
  }
}
```

## Database

| Variable | Required | Default | Example | Surface | Log classification |
|----------|----------|---------|---------|---------|-------------------|
| `ARCFLOW_POSTGRESQL_URL` | Yes for server runs | none | `postgres://user:pass@host:5432/arcflow` | Server, Relay, CLI migrate | **Never log** (contains password) |
| `ARCFLOW_PG_MAX_CONNECTIONS` | No | `10` | `20` | Server pool | Safe |

Pool sizing rule: `(replicas × limit) < Postgres max_connections`.

## Vector store and embeddings

| Variable | Required | Default | Example | Surface | Log classification |
|----------|----------|---------|---------|---------|-------------------|
| `ARCFLOW_QDRANT_URL` | For vector/RAG | none | `http://qdrant:6333` | Server | Safe |
| `ARCFLOW_EMBEDDING_PROVIDER` | Prod with RAG | none | `openai/text-embedding-3-small` | Server | Safe |
| `ARCFLOW_QDRANT_HYBRID` | No | unset/false | `true` | Hybrid retrieval | Safe |
| `COHERE_API_KEY` | When rerank enabled | none | Cohere API key | Rerank provider | **Never log** |

Do not use `stub` embedding provider in production when knowledge ingest or vector agents are enabled.

## Server runtime

| Variable | Required | Default | Example | Surface | Log classification |
|----------|----------|---------|---------|---------|-------------------|
| `ARCFLOW_PORT` | No | `8080` | `8080` | Server listen | Safe |
| `ARCFLOW_CORS_ORIGINS` | No | empty | `https://app.example.com,https://admin.example.com` | Server CORS | Safe |
| `ARCFLOW_DEBUG` | No | false | `true` (dev only) | Debug routes | Safe |
| `ARCFLOW_WEBHOOK_SECRET` | For external callbacks | none | 32+ byte secret | HMAC verify | **Never log** |
| `ARCFLOW_RELAY_PUBLIC_URL` | No | none | `https://relay.example.com` | Admin site responses | Safe |

## Relay

| Variable | Required | Default | Example | Surface | Log classification |
|----------|----------|---------|---------|---------|-------------------|
| `ARCFLOW_UPSTREAM_URL` | Yes | none | `http://arcflow-server:8080` | Relay | Safe |
| `ARCFLOW_RELAY_PORT` | No | `8090` | `8090` | Relay listen | Safe |
| `ARCFLOW_RELAY_SITES_JSON` | BYO Relay | none | inline sites array | Relay static config | **Never log** (contains tokens) |

Relay may share `ARCFLOW_POSTGRESQL_URL` with server for dynamic sites.

## LLM providers

| Variable | Required | Default | Example | Surface | Log classification |
|----------|----------|---------|---------|---------|-------------------|
| `OPENAI_API_KEY` | When using OpenAI | none | `sk-...` | Provider | **Never log** |
| `ANTHROPIC_API_KEY` | When using Anthropic | none | `sk-ant-...` | Provider | **Never log** |
| `GEMINI_API_KEY` | When using Gemini | none | Google AI key | Provider | **Never log** |

Provider selection is per-agent in RCS via `ProviderConfig.api_key_env`.

## Observability (alpha, FP-4)

| Variable | Required | Default | Example | Surface | Log classification |
|----------|----------|---------|---------|---------|-------------------|
| `ARCFLOW_OTEL_ENABLED` | No | false | `true` | OTel init | Safe |
| `ARCFLOW_OTLP_ENDPOINT` | When OTel on | none | `http://otel-collector:4317` | OTLP export | Safe |

See [OpenTelemetry guide](../guides/observability/opentelemetry.md).

## CLI migrate

| Variable | Required | Purpose |
|----------|----------|---------|
| `ARCFLOW_POSTGRESQL_URL` | Yes | `arcflow migrate up` / `validate` |

## Frontend (static SDK, not server env)

These are build-time Vite variables, documented for operator handoff:

| Variable | Purpose |
|----------|---------|
| `VITE_ARCFLOW_RELAY_URL` | Public Relay base URL |
| `VITE_ARCFLOW_SITE_ID` | Site id from admin create |
| `VITE_ARCFLOW_SITE_TOKEN` | One-time site token |

Dashboard dev (private repo): `ARCFLOW_ADMIN_URL`, admin key in BFF only.

## Critical production triple

Minimum for a working server deployment:

1. `ARCFLOW_SERVER_API_KEY` and `ARCFLOW_ADMIN_API_KEY`
2. `ARCFLOW_POSTGRESQL_URL`
3. At least one LLM key (`OPENAI_API_KEY`, etc.)

## Security checklist for operators

- Store secrets in a secret manager or orchestrator secrets, not in git.
- Rotate keys on schedule or on personnel change. See [Token rotation](../operator/token-rotation.md).
- Restrict `.env` file permissions (`chmod 600`).
- Never log environment dumps in production support bundles.

## Related pages

- [Server deployment](server-deployment.md)
- [API key management](../security/api-key-management.md)
- [Production checklist](production-checklist.md)
