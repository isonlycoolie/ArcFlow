# 01 Embedded SDK vs server


## Before you start

You should have completed [First workflow in five minutes](../first-workflow-in-five-minutes.md) or [Python quickstart](../quickstart-python.md). Those pages show embedded SDK execution: Python declares agents, the in-process Rust runtime executes them, and no separate server process is required.

## Concept

ArcFlow exposes one execution engine (`arcflow-core`) through two common integration shapes.

**Embedded SDK** loads the runtime inside your application process. Python or TypeScript serializes workflow definitions to the ArcFlow workflow specification, calls the native binding, and reads `WorkflowResult` in memory. Traces live in the in-process store until evicted. This path suits scripts, notebooks, backend services that already run Python or Node, and local development.

**Server API** runs the same engine inside `arcflow-server`, a Rust HTTP service backed by PostgreSQL. Callers submit JSON to `POST /v1/runs`, poll `GET /v1/runs/{id}`, and fetch traces over HTTP. This path suits polyglot backends, cron jobs, browser-facing products that must not embed the full SDK, and any workflow that needs durable pause and resume (HITL, external callbacks, registry semver resolution).

The SDK can target either shape. Passing `runtime="http://localhost:8080"` to `Workflow()` keeps your declaration code identical while shifting execution to the server.

| Concern | Embedded SDK | Server API |
|---------|--------------|------------|
| Postgres required | No | Yes for `POST /v1/runs` |
| HITL and external callbacks | Limited without server + recovery | Full support with `recovery_enabled` |
| Workflow registry / semver publish | No | Yes |
| Static product (Relay, `runPublished`) | No | Yes |
| LLM keys in app env | App process | Server process |
| Trace access | `workflow.trace()` locally | `GET /v1/runs/{id}/trace` |
| Operational surface | Your app only | Server, Postgres, optional Relay |

## Example

Same workflow, two runtimes. Embedded:

```python
from arcflow import Agent, Workflow

wf = Workflow("demo")
wf.step(Agent(name="writer", role="author", instructions="Summarize in one sentence."))
result = wf.run("renewable energy")
print(result.output)
```

Server-backed SDK (server must be running):

```python
import os

os.environ["ARCFLOW_SERVER_API_KEY"] = "dev-secret"

from arcflow import Agent, Workflow

wf = Workflow("demo", runtime="http://localhost:8080")
wf.step(Agent(name="writer", role="author", instructions="Summarize in one sentence."))
result = wf.run("renewable energy")
print(result.output)
```

Raw HTTP without any SDK is documented in [Server API quickstart](../quickstart-server-api.md).

## Verify

| Check | Embedded | Server |
|-------|----------|--------|
| Stub run completes | Yes, no Docker | Yes, after `docker compose -f docker/docker-compose.server.yml up -d` |
| `result.status` | `completed` (lowercase) | `completed` on SDK; `Completed` on raw HTTP poll |
| Postgres | Not used | Required; `/ready` returns 503 if degraded |

If the server path returns **503** on create, Postgres is unreachable or migrations have not finished. Wait for `/ready` to return **200** before retrying.

## Next

[02 Server API first run](02-server-api-first-run.md) starts the Docker stack and walks through curl against `POST /v1/runs` without the SDK.
