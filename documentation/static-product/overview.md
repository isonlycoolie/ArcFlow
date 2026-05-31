
# Static product overview

The ArcFlow static product is the combination of **admin API**, **workflow registry**, **arcflow-relay**, and **`@arcflow/static`** browser SDK. Together they deliver production chat widgets and published workflows without embedding LLM API keys or full agent definitions in frontend JavaScript bundles.

## Three-tier model

```text
Operator (admin key)
  → POST /v1/admin/sites
  → POST .../knowledge/ingest
  → POST .../workflows/chat/publish

Developer (frontend)
  → embed Relay URL + site token (build-time env)
  → runPublished("chat", "^1.0.0", message)

Browser
  → Relay → Server → arcflow-core
  → poll status / trace (SSE deferred FP-2)
```

| Tier | Actor | Tools |
|------|-------|-------|
| 1 | Operator | Admin API, migrate, ingest, publish |
| 2 | Frontend dev | `@arcflow/static`, Vite env injection |
| 3 | End user | Chat UI calling Relay only |

## Components

| Component | Role |
|-----------|------|
| `arcflow-server` | Registry, runs, admin routes, Postgres |
| `arcflow-relay` | Origin + rate limit + site token auth |
| `@arcflow/static` | Browser client (`ArcFlowClient`, `runPublished`) |
| Qdrant + embedding provider | Knowledge for RAG chat (when configured) |

Relay is stateless. Postgres holds sites, runs, and registry versions.

## Typical operator flow

1. Create site, save one-time `site_token`.
2. Ingest knowledge into site KB namespace.
3. Publish chat workflow version (e.g. `1.0.1`).
4. Hand Relay URL and token to frontend team via secure channel.
5. Frontend calls `runPublished("chat", "^1.0.0", userMessage)`.

Step detail: [site-lifecycle.md](site-lifecycle.md), [knowledge-and-publish.md](knowledge-and-publish.md).

## Typical developer flow

```typescript
import { ArcFlowClient } from "@arcflow/static";

const client = new ArcFlowClient({
  baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
  mode: "relay",
  siteId: import.meta.env.VITE_ARCFLOW_SITE_ID,
});

const result = await client.runPublished("chat", "^1.0.0", "What is your refund policy?");
console.log(result.output);
```

Example app: `examples/static/chat-rag/`.

## What static product does not do

- **Server SSE (FP-2):** no `GET /v1/runs/{id}/events` on server; use trace polling for streaming UX.
- **Operator dashboard UI (FP-3.01):** OSS spec exists; UI in private ArcFlow-Dashboard repo.
- **Inline workflow from browser** when `allow_inline: false` (recommended production default).

## Maturity

Static product (Relay + admin + static SDK) is **production** per maturity matrix (2026-05-31). Graph and RAG features are available when published workflows and Qdrant are configured.

## Related pages

| Page | Topic |
|------|-------|
| [browser-sdk-api.md](browser-sdk-api.md) | SDK reference |
| [security-model.md](security-model.md) | Browser exposure rules |
| [relay/overview.md](../relay/overview.md) | Proxy behavior |
| [server/http-api-reference.md](../server/http-api-reference.md) | Admin routes |
