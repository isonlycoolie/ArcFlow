
# Self-hosted security

Practical security considerations for self-hosted ArcFlow deployments. Complements [Production checklist](../deployment/production-checklist.md) with network, data store, and operational hygiene guidance.

Deployment guides: [Server deployment](../deployment/server-deployment.md), [Deployment overview](../deployment/overview.md#meta-repo-layout).

## Network boundaries

| Service | Exposure |
|---------|----------|
| `arcflow-server` | Internal or behind LB; HTTPS public only |
| `arcflow-relay` | Public HTTPS for browser clients |
| Postgres | Private network; no public port |
| Qdrant | Private network; enable Qdrant auth/TLS for multi-tenant hosts |
| Admin API | BFF or VPN; not open to Internet without strong auth |

External callback integrators must reach server HTTPS from their network; document firewall allowlists.

## Container hardening

Official Dockerfiles run as non-root user `arcflow` (uid 1000). Verify in production:

```bash
docker inspect --format '{{.Config.User}}' arcflow-server
```

Do not override `USER root` in derived images.

## HTTPS termination

Terminate TLS at load balancer, ingress, or reverse proxy (nginx, Caddy, cloud LB). Do not serve API keys over plain HTTP across untrusted networks.

Relay and server may use HTTP inside a private Docker network; edge must be TLS.

## Postgres security

| Practice | Detail |
|----------|--------|
| TLS | Use `sslmode=require` (or stricter) for managed Postgres |
| Credentials | Unique password per environment; rotate with personnel change |
| Least privilege | DB user with DDL only for migrate job; app user without superuser |
| Backups | Encrypted backups; test restore |

Connection URL is **never log** classified.

## Qdrant security

Default compose exposes port 6333 for development. In production:

- Bind Qdrant to internal network only
- Enable Qdrant API key or front with authenticated proxy if multi-tenant

Vector data may contain embedded knowledge from site ingest; treat as confidential.

## Secrets management

| Do | Do not |
|----|--------|
| Inject secrets at runtime from secret manager | Commit `.env` to git |
| Use `openssl rand -hex 32` for keys | Reuse dev keys in prod |
| Separate admin and server keys | Share one key for all routes |
| Rotate on schedule | Log key values in support tickets |

## Log hygiene

| Never log | Why |
|-----------|-----|
| Request bodies on `/v1/runs` | Contains user input |
| External callback raw POST | May contain PII |
| Authorization headers | Contains API keys |
| Ingest text from admin API | Operator-provided content |

Structured logs may include `run_id`, `site_id`, HTTP status, duration.

Align logs with the trace data policy: do not duplicate trace exports with richer content. See [Trace data policy compliance](sec-1-compliance.md).

## CORS and browser direct access

Restrict `ARCFLOW_CORS_ORIGINS` to known admin or dev origins. Production chat should use Relay, not browser-direct server calls with runtime key.

## Webhook and external integrations

Set `ARCFLOW_WEBHOOK_SECRET` before enabling external bindings. See [Webhook HMAC](webhook-hmac.md).

## Debug and development flags

| Flag | Production |
|------|------------|
| `ARCFLOW_DEBUG` | unset or `false` |
| `ARCFLOW_EMBEDDING_PROVIDER=stub` | avoid when RAG live |

## Compliance data in knowledge ingest

Operators may ingest PII into site knowledge namespaces. Your organization owns data retention and lawful basis for that content. Dashboard and BFF must not forward ingest bodies to third-party analytics.

## Incident response hints

| Event | First steps |
|-------|-------------|
| Site token leak | Rotate token; review origin allowlist; check usage tables |
| Admin key leak | Rotate admin key; audit admin API access logs |
| Server key leak | Rotate server key; review run history for anomaly |

## Related pages

- [API key management](api-key-management.md)
- [Relay security model](relay-security-model.md)
- [Production checklist](../deployment/production-checklist.md)
