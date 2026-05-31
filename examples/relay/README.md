# Relay Deployment Examples

## Problem

Production static sites must **never** hold LLM API keys or admin credentials. **ArcFlow Relay** sits between the browser and `arcflow-server`:

- Validates **Origin** against site allowlist
- Applies **rate limits** per site
- Proxies runs and trace reads with a **site token**

Teams that cannot use ArcFlow-hosted relay need a **BYO Docker** deployment on their own infrastructure.

## Examples

| Path | Use case |
|------|----------|
| [`byo-docker/`](byo-docker/) | Self-hosted Relay + compose wiring to upstream server |

## Prerequisites

- Running `arcflow-server` with sites provisioned
- Site token and allowed origins configured (dashboard or admin API)

## Quick start

See [`byo-docker/README.md`](byo-docker/README.md):

```bash
cd examples/relay/byo-docker
docker compose up
```

## Verify

- Browser request from allowed origin: `201` run created
- Wrong origin: `403`
- Exceed `rate_limit_rpm`: `429`

## Production notes

- Terminate TLS at your edge; Relay sees HTTPS from CDN
- Pin Relay and server versions together in meta-repo releases
- See [static/](../static/) for frontend `runPublished()` wiring

## Related docs

- [capabilities reference §14](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md)
- [Relay R1 exit criteria](../../ArcFlow_Improvement_Plans/arcflow-static-product-vision/09-relay-r1-exit-criteria.md)
