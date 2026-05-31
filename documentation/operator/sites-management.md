**Audience:** `[operator]`

# Sites management

Day-to-day operator guide for ArcFlow static product sites: create, configure, ingest knowledge, publish chat, embed credentials, and monitor usage. Assumes `arcflow-server`, `arcflow-relay`, Postgres, and Qdrant are deployed.

The operator dashboard UI is **deferred (FP-3.01)**. Use this guide with the [Admin API reference](admin-api-reference.md), curl, or OSS shell scripts until the private [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git) passes exit criteria.

## Prerequisites

| Item | Value |
|------|-------|
| Admin API base URL | e.g. `https://api.example.com` |
| Admin key | `ARCFLOW_ADMIN_API_KEY` in your shell or BFF |
| Relay public URL | Returned on site create or `ARCFLOW_RELAY_PUBLIC_URL` |

Export for scripts:

```bash
export ARCFLOW_ADMIN_URL=https://api.example.com
export ARCFLOW_ADMIN_API_KEY=your-admin-key
```

## Create a site

```bash
curl -sf -X POST "$ARCFLOW_ADMIN_URL/v1/admin/sites" \
  -H "Authorization: Bearer $ARCFLOW_ADMIN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "Acme Support",
    "allowed_origins": ["https://www.acme.com"],
    "rate_limit_rpm": 60,
    "allow_inline": false,
    "default_workflow_name": "chat"
  }'
```

Save the response fields:

| Field | Where it goes |
|-------|---------------|
| `site_id` | `VITE_ARCFLOW_SITE_ID` |
| `relay_url` | Derive `VITE_ARCFLOW_RELAY_URL` (base through `/v1/sites/{id}`) |
| `site_token` | `VITE_ARCFLOW_SITE_TOKEN` (one-time; never commit) |
| `kb_namespace` | Informational; server manages namespace |

OSS equivalent: `bash scripts/static-provision-site.sh`.

## Configure origins and rate limits

Before marking a site production-ready, ensure at least one HTTPS origin is configured.

```bash
curl -sf -X PATCH "$ARCFLOW_ADMIN_URL/v1/admin/sites/s_abc123" \
  -H "Authorization: Bearer $ARCFLOW_ADMIN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "allowed_origins": ["https://www.acme.com", "https://support.acme.com"],
    "rate_limit_rpm": 120
  }'
```

Relay rejects run requests when the browser `Origin` header does not match the allowlist.

## Ingest knowledge

Paste or upload FAQ and documentation text. The server chunks and embeds into the site namespace.

```bash
curl -sf -X POST "$ARCFLOW_ADMIN_URL/v1/admin/sites/s_abc123/knowledge/ingest" \
  -H "Authorization: Bearer $ARCFLOW_ADMIN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Password reset: click Forgot Password on the login page.",
    "key": "faq-password"
  }'
```

Expect `chunks_ingested > 0`. Re-ingest with the same `key` overwrites prior chunks for that key.

Requires real embedding provider (`ARCFLOW_EMBEDDING_PROVIDER`, not `stub`).

## Publish chat workflow

Set instructions and publish a semver version to the registry:

```bash
curl -sf -X POST "$ARCFLOW_ADMIN_URL/v1/admin/sites/s_abc123/workflows/chat/publish" \
  -H "Authorization: Bearer $ARCFLOW_ADMIN_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "instructions": "Answer only from ingested knowledge. Say when unsure.",
    "version": "1.0.0"
  }'
```

Frontend static SDK:

```typescript
import { runPublished } from "@arcflow/static";

const result = await runPublished("chat", "^1.0.0", userMessage, {
  mode: "relay",
  relayUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  siteId: import.meta.env.VITE_ARCFLOW_SITE_ID,
  siteToken: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
});
```

## Embed credentials in frontend

Production static sites inject env at build time:

```bash
VITE_ARCFLOW_RELAY_URL=https://relay.example.com/v1/sites/s_abc123
VITE_ARCFLOW_SITE_ID=s_abc123
VITE_ARCFLOW_SITE_TOKEN=st_live_xxxxxxxx
```

Never embed `ARCFLOW_ADMIN_API_KEY` or LLM keys in the browser bundle.

## Verify end-to-end

```bash
bash scripts/static-smoke.sh
```

Manual checks:

1. Chat from allowed origin succeeds.
2. Same request from wrong origin returns 403 at Relay.
3. `GET .../runs/{id}/trace` contains metadata events only (SEC-1).

## Monitor usage

Daily usage aggregates land in `arcflow_site_usage_daily` (migration 000007). Tier 2 dashboard charts read this table. Until dashboard ships, query Postgres directly for run counts per site if needed.

Watch Relay logs for 429 rate limit responses.

## Typical operator flow

1. Create site, copy env vars.
2. Ingest knowledge documents.
3. Write chat instructions, publish workflow.
4. Patch origins to match deployed frontend URL.
5. Smoke test from production origin.

Matches `dashboard/spec/02-information-architecture.md` default operator flow.

## Related pages

- [Admin API reference](admin-api-reference.md)
- [Token rotation](token-rotation.md)
- [Relay deployment](../deployment/relay-deployment.md)
- [Knowledge ingestion guide](../guides/memory-and-rag/knowledge-ingestion.md)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §13.1; [dashboard/spec/02-information-architecture.md](../../dashboard/spec/02-information-architecture.md), [03-admin-api-contract.md](../../dashboard/spec/03-admin-api-contract.md).
