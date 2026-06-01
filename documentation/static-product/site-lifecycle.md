
# Site lifecycle

A **site** is the unit of isolation for the static product: one frontend origin set, one rate limit, one knowledge namespace, one site token, and one Relay URL. Operators manage sites through admin API routes authenticated with `ARCFLOW_ADMIN_API_KEY`.

## 1. Create site

```bash
curl -s -X POST http://localhost:8080/v1/admin/sites \
  -H "Authorization: Bearer dev-admin" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "Acme Support",
    "allowed_origins": ["https://www.acme.com"],
    "rate_limit_rpm": 60,
    "allow_inline": false,
    "default_workflow_name": "chat",
    "chat_instructions": "Answer from knowledge only."
  }'
```

Example response:

```json
{
  "site_id": "site_abc123",
  "relay_url": "http://localhost:8090/v1/sites/site_abc123",
  "site_token": "st_live_xYz789...",
  "kb_namespace": "site-site_abc123-kb"
}
```

**The `site_token` is shown once.** Store it in a secret manager or password vault. Do not commit to git.

## 2. Embed in frontend

Inject at build time (Vite example):

```bash
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/site_abc123
VITE_ARCFLOW_SITE_ID=site_abc123
VITE_ARCFLOW_SITE_TOKEN=st_live_xYz789...
```

```typescript
const client = new ArcFlowClient({
  baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
  mode: "relay",
  siteId: import.meta.env.VITE_ARCFLOW_SITE_ID,
});
```

Use CI secret stores (GitHub Actions secrets, Doppler, etc.) for production tokens.

## 3. Read site configuration

```bash
curl -s "http://localhost:8080/v1/admin/sites/site_abc123" \
  -H "Authorization: Bearer dev-admin"
```

Returns origins, rate limit, defaults. Token hash is not reversible; you cannot retrieve the plain token again.

## 4. Update origins and limits

When launching a staging domain or changing traffic expectations:

```bash
curl -s -X PATCH "http://localhost:8080/v1/admin/sites/site_abc123" \
  -H "Authorization: Bearer dev-admin" \
  -H "Content-Type: application/json" \
  -d '{
    "allowed_origins": ["https://www.acme.com", "https://staging.acme.com"],
    "rate_limit_rpm": 120
  }'
```

Relay enforces updated origins on the next request (when site store is synced). BYO Relay JSON configs require redeploy.

## 5. Rotate token

If a token leaks or an employee with access leaves:

```bash
curl -s -X POST "http://localhost:8080/v1/admin/sites/site_abc123/tokens/rotate" \
  -H "Authorization: Bearer dev-admin"
```

Response includes a new `site_token`. Update all frontend deployments. Old token stops working immediately.

## 6. Decommission

Archive site by revoking token, removing origins, and unpublishing workflows as needed. Postgres rows remain for audit unless operational policy deletes them.

## Checklist

| Step | Done when |
|------|-----------|
| Site created | `site_id` and `relay_url` recorded |
| Token stored securely | Not in repo |
| Origins match deploy URLs | Relay 403 tests pass |
| Knowledge ingested | RAG answers cite KB (see knowledge guide) |
| Chat workflow published | `runPublished("chat", "^1.0.0", ...)` succeeds |
| Smoke script green | `scripts/static-smoke.sh` or manual curl |

## Related pages

- [knowledge-and-publish.md](knowledge-and-publish.md)
- [relay/origin-and-rate-limiting.md](../relay/origin-and-rate-limiting.md)
- [static-product/security-model.md](security-model.md)
- [HTTP API reference](../server/http-api-reference.md) (admin routes under `/v1/admin/*`)
