
# API key management

ArcFlow uses a two-tier API key model plus optional scoped runtime keys and per-site tokens. This guide covers generation, storage, rotation, and what each credential can access.

## Key tiers

| Credential | Protects | Typical location |
|------------|----------|------------------|
| `ARCFLOW_SERVER_API_KEY` | `/v1/runs`, registry, trace, approve, external | Server env, backend services |
| `ARCFLOW_ADMIN_API_KEY` | `/v1/admin/*` | BFF env, CI, never browser |
| Scoped runtime key | Subset of workflows via `ARCFLOW_STATIC_RUNTIME_KEYS` | Relay upstream, automation |
| Site token | Relay `/v1/sites/{id}/*` | Frontend build env (`VITE_*`) |

## Generation

Use cryptographically random keys:

```bash
openssl rand -hex 32
```

Minimum practical length: 32 hex characters (256 bits). Use distinct values for server and admin keys.

Do not use compose defaults (`dev-secret`, `dev-admin`) outside local development.

## Scoped runtime keys

JSON in `ARCFLOW_STATIC_RUNTIME_KEYS`:

```json
{
  "relay-upstream-1": {
    "workflows": ["chat"],
    "publish": false
  }
}
```

Relay calls upstream with the scoped key associated with a site (`upstream_runtime_key` or default). Scoped keys cannot call admin routes (**403**).

## What each key can access

| Action | Server key | Admin key | Scoped key | Site token |
|--------|:----------:|:---------:|:----------:|:----------:|
| POST /v1/runs | Yes | No | If workflow allowed | Via Relay only |
| GET /v1/runs/{id} | Yes | No | Yes | Via Relay |
| Registry publish (runtime) | Yes | No | If `publish: true` | No |
| POST /v1/admin/sites | No | Yes | No | No |
| Knowledge ingest | No | Yes | No | No |
| Relay create run | No | No | No | Yes + Origin |

Enforcement matrix: [API key management](../security/api-key-management.md).

## Storage rules

| Rule | Detail |
|------|--------|
| Environment variables | Primary production pattern |
| Secret managers | Vault, AWS SM, K8s Secrets preferred at scale |
| Never in git | No `.env` commits; pre-commit secret scan |
| Never in frontend | Except site token (by design, bounded by Relay) |
| Never in Docker image layers | Inject at runtime |

## Rotation

| Key | Procedure |
|-----|-----------|
| Site token | [Token rotation](../operator/token-rotation.md) |
| Server key | Rolling server deploy + update Relay upstream |
| Admin key | Update BFF secrets + rolling server deploy |

Rotate on suspected exposure, schedule, or personnel change.

## Logging and support

Classification from [Environment variables reference](../deployment/environment-variables-reference.md):

- **Never log:** server key, admin key, LLM keys, webhook secret, site tokens in support bundles
- **Safe to log:** key id names in config (not values), scoped key map structure without secrets

## Debug endpoints

`ARCFLOW_DEBUG=true` enables `/v1/debug/*` on debug builds. Keep disabled in production. Debug routes are localhost-oriented.

## Related pages

- [Token rotation](../operator/token-rotation.md)
- [Relay security model](relay-security-model.md)
- [Self-hosted security](self-hosted-security.md)
