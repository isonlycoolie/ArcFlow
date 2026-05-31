**Audience:** `[platform]` `[compliance]`

# Production checklist

Actionable readiness checklist before accepting production traffic. Aligns with FP-7 production signoff themes and Sprint 8 security rules. Mark each item verified in your change ticket.

## Authentication and secrets

| # | Check | Verification |
|---|-------|--------------|
| P1 | `ARCFLOW_SERVER_API_KEY` set, ≥ 32 hex chars | `openssl rand -hex 32`; not default `dev-secret` |
| P2 | `ARCFLOW_ADMIN_API_KEY` set, distinct from server key | Separate env var in secret store |
| P3 | No API keys in git, Docker images, or frontend bundles | Secret scan CI; inspect built static assets |
| P4 | LLM keys in environment only | No hardcoded provider keys in RCS JSON |
| P5 | `ARCFLOW_WEBHOOK_SECRET` set if external callbacks used | External POST succeeds with valid HMAC only |

## Database and migrations

| # | Check | Verification |
|---|-------|--------------|
| P6 | `ARCFLOW_POSTGRESQL_URL` points to managed or backed-up Postgres | Connection from server pod |
| P7 | Migrations applied through 000007 | `arcflow migrate validate` |
| P8 | `/ready` returns 200 with `"status":"ready"` | Not `migrations_pending` or `postgres_unavailable` |
| P9 | Connection pool sized for replica count | `(replicas × ARCFLOW_PG_MAX_CONNECTIONS) < max_connections` |
| P10 | Postgres backups scheduled | Restore drill documented |

## Container and process security

| # | Check | Verification |
|---|-------|--------------|
| P11 | Server container runs as non-root (`arcflow` uid 1000) | `docker inspect` User field |
| P12 | Relay container runs as non-root | Same pattern |
| P13 | `ARCFLOW_DEBUG` unset or `false` | No `/v1/debug/*` in prod |
| P14 | Request body limit understood (1 MiB) | Large ingest chunked |

## Network and TLS

| # | Check | Verification |
|---|-------|--------------|
| P15 | HTTPS termination at load balancer or ingress | No plain HTTP from Internet clients |
| P16 | Postgres TLS enabled for managed DB | `sslmode=require` in URL where supported |
| P17 | Qdrant not exposed publicly | Internal network or auth fronting |
| P18 | `ARCFLOW_CORS_ORIGINS` restricted | No wildcard `*` in production |
| P19 | Browser traffic uses Relay, not direct server key | Static SDK `mode: "relay"` |

## Static product (if applicable)

| # | Check | Verification |
|---|-------|--------------|
| P20 | Sites have HTTPS `allowed_origins` | Relay rejects wrong Origin |
| P21 | `allow_inline: false` on production sites | No browser workflow override |
| P22 | Site tokens rotated on schedule | Procedure in [Token rotation](../operator/token-rotation.md) |
| P23 | Real embedding provider configured | Not `stub` for knowledge ingest |
| P24 | Chat workflow published to registry | `POST .../workflows/chat/publish` success |

## Observability and compliance

| # | Check | Verification |
|---|-------|--------------|
| P25 | SEC-1 audit on sample trace | No prompt/completion text in `GET .../trace` |
| P26 | Application logs do not duplicate trace with richer content | Log review |
| P27 | OTel configured if required by policy | `ARCFLOW_OTEL_ENABLED` + collector (FP-4 alpha) |
| P28 | External callback integrators reach server over HTTPS | Network path test |

## Operational smoke

Run after deploy:

```bash
curl -sf http://localhost:8080/health
curl -sf http://localhost:8080/ready

# Runtime smoke (adjust workflow body)
curl -sf -X POST http://localhost:8080/v1/runs \
  -H "Authorization: Bearer $ARCFLOW_SERVER_API_KEY" \
  -H "Content-Type: application/json" \
  -d @examples/minimal-run.json

bash scripts/load-test-runs.sh   # optional capacity check
bash scripts/static-smoke.sh     # static product end-to-end
```

## Deferred items (do not mark as shipped)

| ID | Item |
|----|------|
| FP-3.01 | Operator dashboard UI (private ArcFlow-Dashboard repo) |
| FP-2 | Server SSE `/v1/runs/{id}/events` |
| FP-4 | Stable OTel metrics |
| FP-5.04 | Full `arcflow validate` against schema |

## Sign-off

| Role | Confirms |
|------|----------|
