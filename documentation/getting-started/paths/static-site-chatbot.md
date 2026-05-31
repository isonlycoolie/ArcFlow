# Static site chatbot

**Audience:** `[frontend]` `[operator]`

## Before you start

This path walks through the production static-site pattern end to end: server and Relay via Docker, operator provisioning through the admin API, knowledge ingest, chat workflow publish, and a minimal browser client in `examples/static/chat-rag/`.

You should have completed [Server API quickstart](../quickstart-server-api.md) or be comfortable starting the Docker stack. Node.js 18+ is required for the Vite example.

In production, frontend developers do **not** define agents, memory, or ingest documents in browser code. Operators own knowledge and publish records through the ArcFlow Dashboard or admin API. The browser ships two env vars and calls `runPublished()`.

## Concept

The static product splits responsibilities:

| Who | Responsibility |
|-----|----------------|
| Operator / dashboard user | Create site, upload knowledge, set chat instructions, publish workflow |
| Frontend developer | Paste relay URL and site token, wire chat UI, call `runPublished()` |
| ArcFlow Relay | Holds site token, validates Origin, proxies runs to server |
| arcflow-server | Executes published workflow plus RAG against ingested knowledge |

Browser traffic never carries LLM keys or server admin keys. The site token is scoped to Relay routes for that site.

Published workflows resolve by name and semver range. The default chat workflow name is `chat`. The frontend calls `runPublished("chat", "^1.0.0", userMessage)` with no inline agent JSON in the bundle.

Progressive UI today uses **trace polling**, not server SSE. `GET /v1/runs/{run_id}/events` (SSE) is deferred as **FP-2**. Plan browser streaming around Relay trace poll and `TokenEmitted` metadata events. See [Streaming in the browser](../../guides/streaming/streaming-in-the-browser.md).

## Prerequisites

| Item | Required |
|------|----------|
| Docker Compose v2 | Server + Relay stack |
| Node.js 18+ | Vite example |
| Bash + curl | Operator scripts |
| Repo clone | Commands assume repo root |

## Step 1: Start server and Relay

From the repository root:

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
```

Confirm readiness:

```bash
curl -sf http://localhost:8080/health
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

Wait for **200** on `/ready` before provisioning. Postgres and migrations must be healthy.

## Step 2: Provision site (admin API)

Use the provision script (equivalent to Dashboard → Sites → Create):

```bash
bash scripts/static-provision-site.sh
```

The script prints values to copy:

```text
SITE_ID=s_...
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/s_...
VITE_ARCFLOW_SITE_TOKEN=st_...
```

Export `SITE_ID` for the next steps:

```bash
export SITE_ID=s_...   # from script output
```

The script registers `http://localhost:5173` as an allowed origin by default. Override with `ARCFLOW_SITE_ORIGIN` if your dev server uses another port.

Manual equivalent:

```bash
curl -sf -X POST "http://localhost:8080/v1/admin/sites" \
  -H "X-ArcFlow-Admin-Key: dev-admin" \
  -H "Content-Type: application/json" \
  -d '{"display_name":"Static Dev Site","allowed_origins":["http://localhost:5173"]}'
```

When the ArcFlow Dashboard is deployed in your environment, use Sites → Create and copy relay URL and site token from the UI instead.

## Step 3: Ingest knowledge

Sample content lives in `examples/static/chat-rag/kb.txt`. Ingest via script:

```bash
