# Relay BYO deployment example

**Audience:** `[operator]` `[platform]`

This walkthrough self-hosts ArcFlow Relay beside your own `arcflow-server` using Docker Compose. Browser clients use site tokens and Origin checks identical to managed Relay.

Primary example: [`examples/relay/byo-docker/`](../../examples/relay/byo-docker/).

## What this example demonstrates

Bring-your-own (BYO) Relay lets operators run the browser proxy in their VPC while upstream execution stays on `arcflow-server`. Site configuration (allowed origins, rate limits, upstream runtime keys) loads from `ARCFLOW_RELAY_SITES_JSON`.

## Prerequisites

| Item | Required |
|------|----------|
| `arcflow-server` reachable | Default `http://localhost:8080` |
| Docker Compose v2 | For `compose.yml` in example dir |
| Published workflow | e.g. `chat` workflow on server |
| Frontend | Static app with Relay env vars |
| Tutorial track | [Track F](../tutorials/track-f-static-product.md) (Relay section) |

## Step 1: Start upstream server

From repository root:

```bash
docker compose -f docker/docker-compose.server.yml up -d
curl -sf http://localhost:8080/ready
```

## Step 2: Configure Relay sites JSON

Export upstream URL and site definition (from [`README.md`](../../examples/relay/byo-docker/README.md)):

```bash
export ARCFLOW_UPSTREAM_URL=http://host.docker.internal:8080
export ARCFLOW_RELAY_SITES_JSON='[{
  "id": "s_dev",
  "display_name": "Dev",
  "allowed_origins": ["http://localhost:5173"],
  "rate_limit_rpm": 60,
  "allow_inline": false,
  "default_workflow_name": "chat",
  "kb_namespace": "site-s_dev-kb",
  "upstream_runtime_key": "dev-secret",
  "token": "st_live_devtoken"
}]'
```

On Linux Docker, replace `host.docker.internal` with the host gateway IP if needed.

## Step 3: Start BYO Relay

```bash
cd examples/relay/byo-docker
docker compose -f compose.yml up --build
```

Verify health:

```bash
curl -sf http://localhost:8090/health
```

## Step 4: Point static frontend at BYO Relay

```bash
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/s_dev
VITE_ARCFLOW_SITE_TOKEN=st_live_devtoken
```

Run [`examples/static/chat-rag/`](../../examples/static/chat-rag/) or your own static bundle against port 8090.

## Relay routes

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness |
| POST | `/v1/sites/{site_id}/runs` | Create run |
| GET | `/v1/sites/{site_id}/runs/{run_id}` | Poll status |
| GET | `/v1/sites/{site_id}/runs/{run_id}/trace` | Fetch trace |

Auth: `Authorization: Bearer {site_token}` plus allowed `Origin` header on browser requests.

## Expected output

Health endpoint returns 200. Browser chat through Relay creates runs on upstream server with site-scoped runtime key. Rate limiting applies per `rate_limit_rpm` when configured.

Pass criteria:

| Check | Expected |
|-------|----------|
| `/health` | 200 |
| POST run from allowed origin | 201 with `run_id` |
| POST from disallowed origin | Rejected at Relay |
| Upstream receives run | Visible in server logs or `GET /v1/runs` with server key |

## Trace events you should see

Retrieve via Relay trace route or server admin tools. Expect standard workflow lifecycle events on published workflow runs (`WorkflowStarted`, `StepCompleted`, `WorkflowCompleted`, optional `MemoryRetrieved` for RAG sites).

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Relay cannot reach upstream | Wrong `ARCFLOW_UPSTREAM_URL` from container | Use host gateway or shared Docker network |
| 401 from upstream | `upstream_runtime_key` mismatch | Match `ARCFLOW_SERVER_API_KEY` on server |
| 403 Origin | Browser origin not in JSON | Add exact origin string to `allowed_origins` |
| Inline workflow rejected | `allow_inline: false` | Publish workflow on server; use `runPublished` |

## Related

| Resource | Link |
|----------|------|
| Static chat widget | [static-chat-widget.md](static-chat-widget.md) |
| Static examples index | [`examples/static/README.md`](../../examples/static/README.md) |
| Tutorial track | [Track F](../tutorials/track-f-static-product.md) |

**Source:** [`examples/relay/byo-docker/README.md`](../../examples/relay/byo-docker/README.md), [`examples/relay/byo-docker/compose.yml`](../../examples/relay/byo-docker/compose.yml); capabilities reference §25, §28 Track F; `docker/docker-compose.server.yml`.
