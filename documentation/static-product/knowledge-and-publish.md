
# Knowledge and publish

Static product chat workflows combine **ingested knowledge** (vector store) with **semver-published workflow definitions** (registry). Operators use admin routes; browsers call `runPublished("chat", "^1.0.0", message)` without seeing agent JSON.

## Knowledge ingest

Add text to the site's KB namespace (Qdrant collection derived from `kb_namespace`):

```bash
curl -s -X POST "http://localhost:8080/v1/admin/sites/site_abc123/knowledge/ingest" \
  -H "Authorization: Bearer dev-admin" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Refunds are available within 30 days of purchase with receipt.",
    "key": "faq-refunds"
  }'
```

Server chunks, embeds, and upserts per `MemoryChunkingConfig`. Requires `ARCFLOW_QDRANT_URL` and a non-stub embedding provider in production.

| Field | Purpose |
|-------|---------|
| `text` | Raw document content |
| `key` | Logical document id for re-ingest / replace |

Re-ingest with the same `key` to update content. See [guides/memory-and-rag/knowledge-ingestion.md](../guides/memory-and-rag/knowledge-ingestion.md) for chunk tuning.

## Publish chat workflow

Publish binds instructions and version to the registry workflow name `chat` (or site default):

```bash
curl -s -X POST "http://localhost:8080/v1/admin/sites/site_abc123/workflows/chat/publish" \
  -H "Authorization: Bearer dev-admin" \
  -H "Content-Type: application/json" \
  -d '{
    "instructions": "Answer only from ingested knowledge. Say when unsure.",
    "version": "1.0.1"
  }'
```

The server builds an RCS workflow with RAG agent tooling and registers `chat@1.0.1`. Prior versions remain addressable by exact version.

## Semver resolution at runtime

Browser SDK:

```typescript
await client.runPublished("chat", "^1.0.0", userMessage);
```

Relay proxies to server:

```json
{
  "workflow_ref": { "name": "chat", "version": "^1.0.0" },
  "input": "What is your refund policy?"
}
```

Server resolves highest matching non-deprecated version (e.g. `1.0.2` if published):

```bash
curl -s "http://localhost:8080/v1/workflows/chat/resolve?range=%5E1.0.0" \
  -H "Authorization: Bearer dev-secret"
```

Response:

```json
{
  "name": "chat",
  "version": "1.0.2",
  "definition": { }
}
```

## Alias `latest` (optional)

Operators can pin a friendly alias:

```bash
curl -s -X POST "http://localhost:8080/v1/workflows/chat/aliases/latest" \
  -H "Authorization: Bearer dev-secret" \
  -H "Content-Type: application/json" \
  -d '{"version": "1.0.2"}'
```

Static SDK semver ranges (`^1.0.0`) are preferred over alias strings in browser code for predictable upgrades.

## End-to-end operator sequence

```text
1. POST /v1/admin/sites           → site_id, token, kb_namespace
2. POST .../knowledge/ingest      → FAQ / docs in Qdrant
3. POST .../workflows/chat/publish → chat@1.0.0
4. Frontend runPublished          → resolve ^1.0.0 → run
5. Trace MemoryRetrieved          → chunk_count, total_bytes (SEC-1)
```

## Smoke scripts

Repository reference implementations:

| Script | Purpose |
|--------|---------|
| `scripts/static-provision.sh` | Create site, ingest sample, publish |
| `scripts/static-smoke.sh` | End-to-end run via Relay |

Use in CI after deploy to catch registry or Qdrant misconfiguration early.

## Infrastructure requirements

| Service | Required for |
|---------|--------------|
| Postgres | Sites, registry, runs |
| Qdrant | Vector retrieval in chat |
| Embedding provider env | Ingest and query embeddings |

Stub embedding works for local smoke only, not production RAG quality.

## Related pages

- [site-lifecycle.md](site-lifecycle.md)
- [browser-sdk-api.md](browser-sdk-api.md)
- [guides/workflows/workflow-registry.md](../guides/workflows/workflow-registry.md)
