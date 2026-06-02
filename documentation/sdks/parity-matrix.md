# SDK and surface parity matrix


This matrix compares ArcFlow capabilities across runtime surfaces. Values come from [SDK parity matrix](../sdks/parity-matrix.md), verified 2026-05-31 against `arcflow-core`, `arcflow-server`, and SDK exports.

Use it when answering "can I do X in TypeScript?" or "does Relay support graph workflows?"

## Legend

| Symbol | Meaning |
|--------|---------|
| Y | Supported in this surface |
| N | Not supported |
| partial | Behavior exists; known gap documented |
| pub | Supported via published workflow (`runPublished` in static SDK) |
| stub | Alpha or limited (WASM linear stub) |
| view | Read or visualize only (VS Code) |
| debug | Debug/run from workspace, not production path |
| migrate | CLI applies schema; not a workflow feature |
| proxied | Relay forwards to server capability |
| poll | Client polls instead of push stream |
| Y* | Conditional (see row note) |

## Full matrix

| Capability | Py | TS | Server | Relay | Static | CLI | VSCode | WASM |
|------------|:--:|:--:|:------:|:-----:|:------:|:---:|:------:|:----:|
| Linear workflows | Y | Y | Y | Y | pub | Y | view | stub |
| Graph workflows | Y | Y | Y | Y | pub | N | view | N |
| Tools / LLM | Y | Y | Y | Y | pub | N | debug | stub |
| RAG / vector | Y | Y | Y | Y | pub | N | N | N |
| Recovery / migrate | Y | Y | Y | N | N | migrate | N | N |
| HITL | Y | Y | Y | Y* | poll | N | N | N |
| External callback | Y | Y | Y | N | N | N | N | N |
| SDK streaming | Y | Y | N | N | poll | N | N | N |
| Server SSE | N | N | N | N | N | N | N | N |
| Trace read | Y | Y | GET | GET | GET | trace | timeline | N |
| Workflow registry | N | N | Y | via srv | pub | N | N | N |
| Admin / sites | N | N | Y | N | N | N | N | N |
| Idempotency-Key | N | N | Y | proxied | N | N | N | N |

## Row notes

### Linear workflows

All SDK and server paths execute ordered steps. WASM supports a linear stub for edge experiments only, not production.

### Graph workflows

Python and TypeScript expose graph builders in-process. CLI does not author graphs interactively. VS Code visualizes graph workflow specification definitions.

**Partial recovery (Graph recovery resume):** Graph runs persist checkpoint fields in Postgres, but mid-graph resume dispatch is incomplete. Linear recovery works. Do not plan SLA around graph resume until Graph recovery resume closes.

### Tools / LLM

Python `Agent` accepts `Tool` instances and full tool loop config. TypeScript `Agent` is name/role/instructions only in the binding layer; tools work when defined in the workflow specification sent to server or via Python-authored workflows.

VS Code debug path invokes TS SDK for local runs.

### RAG / vector

Both SDKs export `VectorStore` for ingest/search. Static product uses dashboard-ingested knowledge and published workflows, not in-browser vector APIs.

### Recovery / migrate

SDKs set `recovery_enabled` in exec config. Server persists recovery state. CLI `migrate up` applies Postgres schema required for recovery but does not run workflows.

### HITL

Server supports approve/reject APIs. Relay may proxy approve when scoped site key allows (Y*).

Static SDK polls run status for interrupt resolution rather than holding a long-lived SSE connection.

### External callback

SDKs and server support async external outcome reports with HMAC verification. Relay and static browser client do not receive external callbacks directly.

Python ships `report_outcome()` helper. TypeScript exports types only.

### SDK streaming

`run_stream()` / `runStream()` work in-process in Python and TypeScript. Not available on server HTTP surface.

### Server SSE

**Deferred (streaming deferred):** `GET /v1/runs/{id}/events` is not implemented. Do not document server SSE as shipped. Workarounds: SDK streaming in your backend, poll GET run, or static SDK poll for tokens.

### Trace read

SDKs: `workflow.trace()` after run. Server and Relay: GET trace endpoints. CLI: `arcflow trace <run_id>`. VS Code: timeline view of metadata events.

### Workflow registry

Server stores semver-published workflows. Static SDK calls `runPublished(name, version, input)`. SDKs can `publish()` and `resolve()` when `runtime=` URL is configured.

### Admin / sites

Server-only admin API for static product (sites, knowledge ingest, chat publish). Not exposed in SDK packages.

### Idempotency-Key

Server honors idempotency on mutating routes. Relay forwards the header to upstream.

## Python vs TypeScript SDK detail

| Feature | Python | TypeScript |
|---------|:------:|:----------:|
| `Workflow` linear + graph | Y | Y |
| `Agent` tools | Y | N (binding gap) |
| `Agent` memory / context | Y | N (binding gap) |
| `Tool` class | Y | N |
| Providers OpenAI/Anthropic/Gemini | Y | Y |
| `VectorStore` | Y | Y |
| `run_stream` / `runStream` | Y | Y |
| Recovery + HITL | Y | Y |
| `report_outcome()` | Y | N (types only) |
| `ScheduleManifest` | Y | N |
| LangChain adapter | Y (`arcflow.langchain`) | N |
| Vitest test helpers | N | Y |
| `mapNativeError` | N | Y |

## Related pages

| Page | Content |
|------|---------|
| [Python overview](python/overview.md) | Python SDK capabilities |
| [TypeScript overview](typescript/overview.md) | TypeScript SDK capabilities |
| [Maturity and known gaps](../concepts/maturity-and-known-gaps.md) | Graph recovery resume, Operator dashboard UI |
