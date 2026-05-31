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
