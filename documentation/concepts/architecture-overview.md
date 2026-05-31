**Audience:** `[developer]` `[platform]` `[frontend]` `[operator]`

# Architecture overview

ArcFlow stacks a single Rust execution engine under multiple language and network surfaces. Every path validates RCS v1 JSON, schedules steps, emits SEC-1 traces, and optionally persists recovery state. Understanding the layers makes it easier to place auth, Postgres, and Qdrant in the right tier.

## Layer model

At the top sit the **surfaces**: Python SDK, TypeScript SDK, `arcflow-server`, `arcflow-relay`, static browser SDK (`@arcflow/static`), CLI, VS Code extension, and WASM (alpha). Each surface translates its native API into the same workflow and run envelope the engine expects.

Below that is **RCS v1 JSON**: workflow definitions, agent definitions, run requests, and `exec_config`. Normative schemas live under [contracts/normative/rcs/](../../contracts/normative/rcs/v1.schema.json). See [The RCS contract](the-rcs-contract.md) for how contract-first design keeps SDK and HTTP behavior aligned.

The **engine** (`arcflow-core`) performs validate, schedule, execute, trace, and recover. Entry point for runs is `WorkflowEngine::execute_with_config`. Linear runs use sorted steps; graph runs use the graph scheduler in `workflow/graph/scheduler.rs`.

**Persistence and external services** split by concern:

| Backend | Role |
|---------|------|
| PostgreSQL | Runs, recovery checkpoints, HITL interrupt state, trace persistence, workflow registry, static product sites |
| Qdrant + LLM APIs | Vector memory, embeddings, rerank, provider chat completions |

Postgres is mandatory for `POST /v1/runs` on the server. Embedded SDK runs can skip Postgres unless you enable recovery or registry features. Relay is stateless and forwards to the server with scoped runtime keys.

## Design principles

These principles show up in code review and deployment checklists:

| Principle | Meaning |
|-----------|---------|
| Contract-first | RCS JSON and normative schemas precede surface-specific APIs |
| Typed errors | RCS `ErrorCode` values map to structured HTTP JSON on the server |
| SEC-1 traces | Metadata only in traces: no prompt text, tool args, or chunk content |
| Self-hosted keys | LLM and embedding keys in environment variables, never in browser bundles |
| Recovery optional | `exec_config.recovery_enabled` gates Postgres persistence for resume |

## Static product flow

The static product path serves public websites: a browser talks to Relay, Relay talks to the server, the server executes against LLM and RAG backends. Site tokens and origin allowlists replace exposing server API keys in JavaScript.

**Sequence in prose:**

1. The browser sends `POST /v1/sites/{site_id}/runs` to Relay with a site token and an `Origin` header that matches the site configuration.
2. Relay validates the origin and applies per-site rate limits. On success it forwards `POST /v1/runs` to `arcflow-server` using a scoped runtime key that limits which workflows the site may run.
3. The server resolves the workflow (often via registry semver, e.g. `runPublished("chat", "^1.0.0", message)`), executes through `arcflow-core`, and returns `run_id` and status to Relay, which passes them to the browser.
4. The browser polls trace via Relay: `GET /v1/sites/{site_id}/runs/{run_id}/trace`, which proxies to `GET /v1/runs/{id}/trace` on the server. Events are metadata-only (SEC-1).

Concrete browser API:

```typescript
import { runPublished } from "@arcflow/static";

const result = await runPublished("chat", "^1.0.0", userMessage, {
  mode: "relay",
  relayUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  siteId: import.meta.env.VITE_ARCFLOW_SITE_ID,
  siteToken: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
});
```

Operators provision sites and publish chat workflows through the admin API (`POST /v1/admin/sites`, knowledge ingest, `POST .../workflows/chat/publish`). Details live under [static-product/](../static-product/overview.md).

**Note:** Server-sent events for live run streaming are not implemented (FP-2). Browser UX today polls trace for `TokenEmitted` and completion events.

## Backend integration flow

Backend services integrate directly with `arcflow-server` when they already own auth and tenancy. This path is typical for internal tools and multi-tenant SaaS backends.

**Sequence in prose:**

1. The client sends `POST /v1/runs` with `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>` and optionally `Idempotency-Key` for deduplication within the server window.
2. The server persists a run row in PostgreSQL, then invokes `execute_with_config` on the engine.
3. During execution the engine writes recovery checkpoints and trace events to Postgres when `recovery_enabled` is true. Linear recovery can resume from the last committed step index; graph checkpoint fields exist but full resume dispatch is partial (FP-1.01).
4. The server returns `run_id`, `trace_id`, and initial `status` (often `Running`).
5. The client polls `GET /v1/runs/{run_id}` for terminal status, result payload, errors, or HITL interrupt metadata. Trace export uses `GET /v1/runs/{run_id}/trace`.

Example run request body:

```json
{
  "workflow": { "id": "...", "name": "demo", "execution_mode": "linear", "steps": [] },
  "agents": [{ "id": "...", "name": "a", "role": "r", "instructions": "..." }],
  "input": "user message",
  "exec_config": { "recovery_enabled": true, "workflow_timeout_secs": 300 }
}
```

Alternatively use `"workflow_ref": { "name": "chat", "version": "^1.0.0" }` instead of an inline workflow object, not both.

If Postgres is unavailable, `POST /v1/runs` returns **503**. Health checks: `GET /health` (process up), `GET /ready` (Postgres connected, migrations applied).

## Where to go next

| Question | Page |
|----------|------|
| Which binary or package do I run? | [Surfaces and when to use them](surfaces-and-when-to-use-them.md) |
| Linear vs graph execution | [Execution model](execution-model.md) |
| Trace field rules | [SEC-1 and data safety](sec-1-and-data-safety.md) |
| Production vs deferred | [Maturity and known gaps](maturity-and-known-gaps.md) |

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §2 (Architecture), including static product and backend integration sequence diagrams.
