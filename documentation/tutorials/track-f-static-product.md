# Track F: Static product

**Audience:** `[frontend]` `[operator]`

Track F provisions a static-site chat product: admin setup, knowledge ingest, workflow publish, Relay-backed browser client, and origin enforcement.

## Goal

Create an admin site, ingest knowledge, publish a workflow, embed the static SDK, and run the chat widget from a browser. Verify origin enforcement blocks disallowed origins.

## Prerequisites

| Item | Required |
|------|----------|
| Server + Relay | `docker compose -f docker/docker-compose.server.yml up -d` |
| Node.js | 18+ for Vite example |
| Operator scripts or dashboard | Site, ingest, publish |
| Primary examples | [static-chat-widget](../examples/static-chat-widget.md), [relay-byo-deployment](../examples/relay-byo-deployment.md) |
| App | [`examples/static/chat-rag/`](../../examples/static/chat-rag/) |
| Track B | Helpful for understanding server admin APIs |

Frontend developers do not define agents in browser code in production. Dashboard or admin API owns workflow shape.

## Step 1: Provision site and ingest knowledge

```bash
docker compose -f docker/docker-compose.server.yml up -d
bash scripts/static-provision-site.sh
export SITE_ID=...  # from output
export TEXT_FILE=examples/static/chat-rag/kb.txt
bash scripts/static-ingest-knowledge.sh
bash scripts/static-publish-chat.sh
```

Record relay URL, site token, and allowed origins from site configuration.

Alternative: ArcFlow Dashboard Sites, Knowledge, and Chat tabs when available in your deployment.

## Step 2: Configure frontend env

```bash
cd examples/static/chat-rag
# set VITE_ARCFLOW_RELAY_URL and VITE_ARCFLOW_SITE_TOKEN
npm install
npm run dev
```

Open http://localhost:5173 and send a chat message.

## Step 3: Verify successful run from allowed origin

Browser network panel should show POST to Relay `/v1/sites/{id}/runs` returning **201** with `run_id`. Poll or client callback receives assistant reply.

Optional test:

```bash
pytest examples/static/chat-rag/test_static.py -q
```

## Step 4: Verify origin enforcement

Temporarily remove `http://localhost:5173` from allowed origins (or test from an unlisted origin tool). Retry chat request. Relay should reject before upstream execution.

Re-add origin and confirm chat works again.

Pass criteria:

| Check | Expected |
|-------|----------|
| Allowed origin chat | Success |
| Disallowed origin | Blocked at Relay |
| Bundle | No LLM or server runtime keys, only site token |
| Published name | Client calls `runPublished("chat", "^1.0.0", ...)` |

## Step 5: Optional BYO Relay

Deploy self-hosted Relay per [relay-byo-deployment](../examples/relay-byo-deployment.md) and point env vars at port 8090.

## Expected output

Working chat UI with answers grounded in ingested `kb.txt` content. Operator scripts print site id and tokens once during provision.

## Trace events you should see

Operators fetch trace via Relay or server:

| Event kind | When |
|------------|------|
| `WorkflowStarted` | Published chat run |
| `MemoryRetrieved` | Knowledge hit on question |
| `StepCompleted` | Chat agent step |
| `WorkflowCompleted` | Successful reply |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| CORS error | Origin not listed | Update site allowed origins |
| 401 on Relay | Invalid site token | Re-copy token from provision output |
| Empty answers | Knowledge not ingested | Re-run ingest script |
| Inline Agent in bundle | Wrong dev path | Use publish flow; avoid shipping `main-dev-direct.ts` |

## What you learned

Track F splits responsibilities: operators own knowledge and publish records; frontend developers ship env vars and UI; Relay enforces browser trust boundaries.

## Next tracks

| Track | Focus |
|-------|-------|
| G | Migrations, readiness, CLI trace export |
| Level 3 cert | Full stack deployment project |

**Source:** capabilities reference §28 Track F; [`examples/static/chat-rag/`](../../examples/static/chat-rag/), [`examples/static/README.md`](../../examples/static/README.md); [static-chat-widget](../examples/static-chat-widget.md), [relay-byo-deployment](../examples/relay-byo-deployment.md).
