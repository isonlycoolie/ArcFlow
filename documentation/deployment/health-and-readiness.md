
# Health and readiness

ArcFlow exposes two unauthenticated HTTP probes on the server: liveness (`/health`) and readiness (`/ready`). Use them in Docker HEALTHCHECK, Kubernetes probes, and load balancer health checks.

Implementation: `server/arcflow-server/src/handlers/health.rs`, `ready.rs`.

## GET /health (liveness)

Confirms the process is running and serving HTTP.

**Request:** `GET /health` (no auth)

**Response 200:**

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

| Field | Meaning |
|-------|---------|
| `status` | Always `"ok"` when handler runs |
| `version` | Server crate version from build |

Use for **liveness** probes. A failing liveness probe means restart the pod or container.

Dockerfile built-in check:

```dockerfile
HEALTHCHECK CMD curl -f http://localhost:8080/health || exit 1
```

## GET /ready (readiness)

Confirms the server can accept runtime traffic: Postgres reachable and migrations current.

**Request:** `GET /ready` (no auth)

**Response 200 (ready):**

```json
{
  "status": "ready",
  "version": "0.1.0"
}
```

**Response 200 (Postgres not configured):**

```json
{
  "status": "ready",
  "version": "0.1.0",
  "postgres": "not_configured"
}
```

When `ARCFLOW_POSTGRESQL_URL` is unset, `/ready` returns 200 with `postgres: not_configured`. This suits dev builds but **not** production server deployments that must serve `/v1/runs`.

**Response 503 (degraded):**

```json
{
  "status": "degraded",
  "version": "0.1.0",
  "reason": "postgres_unavailable"
}
```

```json
{
  "status": "degraded",
  "version": "0.1.0",
  "reason": "migrations_pending"
}
```

```json
{
  "status": "degraded",
  "version": "0.1.0",
  "reason": "migration_check_failed"
}
```

| `reason` | Meaning | Operator action |
|----------|---------|-----------------|
| `postgres_unavailable` | Pool ping failed | Check DB connectivity, credentials, TLS |
| `migrations_pending` | Schema behind code | Run `arcflow migrate up` |
| `migration_check_failed` | Could not read migration state | Inspect logs; restore DB if corrupted |

Use for **readiness** probes. Remove instance from load balancer when readiness fails.

## Kubernetes example

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

Gate Ingress or Service traffic on readiness 200 only.

## Load balancer configuration

| Probe | Path | Success | Action on failure |
|-------|------|---------|-------------------|
| Liveness | `/health` | 200 | Restart instance |
| Readiness | `/ready` | 200 | Stop sending new requests |

Do not use `/health` alone for traffic routing in production; the process may be alive while Postgres is down.

## Deploy sequence integration

Recommended order:

1. Start Postgres.
2. Complete `arcflow migrate up`.
3. Start server.
4. Poll `/ready` until 200 (not degraded).
5. Register with load balancer.
6. Start Relay after server ready.

Smoke script:

```bash
curl -sf http://localhost:8080/health
curl -sf http://localhost:8080/ready
```

## Relay health

Relay does not share the same `/ready` semantics as the server. Monitor Relay by upstream availability and successful proxied run creation. If Relay exposes a health route in your build, use it for liveness only.

## Related pages

- [Migrations runbook](migrations-runbook.md)
- [Server deployment](server-deployment.md)
- [Production checklist](production-checklist.md)
