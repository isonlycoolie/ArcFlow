
# What is ArcFlow?

ArcFlow is a self-hosted AI workflow runtime. The Rust engine in `arcflow-core` executes workflows that combine LLM steps, tools, vector RAG, graph DAGs, human-in-the-loop (HITL) gates, and external HTTP callbacks. Python, TypeScript, the HTTP server, Relay, CLI, VS Code, and an alpha WASM build all call the same engine, so a workflow validated in a notebook behaves the same when published to a public chat widget.

The product goal is simple: give teams a single runtime they control, with contracts and traces that compliance can audit, without shipping provider secrets to browsers.

## Who ArcFlow is for

Different readers care about different guarantees. The table below maps personas to what they typically need from the stack.

| Persona | What they need |
|---------|----------------|
| Platform engineer | Shared runtime, recovery, workflow registry, Postgres-backed runs |
| Agent engineer | Multi-agent pipelines, tools, RAG, conditional graph routing |
| Frontend developer | Chat and forms in the browser without CDN or bundle secrets |
| Operator | Sites, knowledge bases, semver publish for published chat workflows |
| Compliance | metadata-only traces, self-hosted API keys, clear auth boundaries |

If you are deciding where to start, platform and agent engineers usually begin with the SDK or server path. Frontend and operator readers usually begin with Relay, the static SDK, and the admin API. Compliance readers should read [Trace data policy](sec-1-and-data-safety.md) before any production deployment.

## What ArcFlow is not

ArcFlow deliberately does not try to be everything in the AI platform space. Treat the following as explicit non-goals:

**ArcFlow Cloud and proprietary hosting.** You run the binaries and data stores. There is no managed ArcFlow SaaS in this repository.

**Regulated product claims.** ArcFlow is not positioned as a medical device, clinical decision support system, or live trading platform. Domain-specific validation and certification remain your responsibility.

**A replacement for your identity layer.** The server exposes API keys and admin keys. Multi-tenant SaaS still needs your auth, tenancy, and billing around `POST /v1/runs`.

**A browser LLM client.** Production static sites use Relay so `OPENAI_API_KEY` and similar values stay on the server. Putting provider keys in frontend bundles is a dev-only anti-pattern.

Knowing these boundaries saves time when evaluating ArcFlow against hosted agent platforms or low-code chat builders.

## Core capabilities in one pass

Workflows are defined in workflow specification JSON (see [Workflow specification](the-rcs-contract.md)). A minimal linear workflow sorts steps by `order` and hands state between agents. Graph mode adds nodes, conditional edges, parallel fan-out, and join nodes for branch merge.

Agents reference provider config via environment variable names (`api_key_env`), not inline secrets. Memory types include session, shared, persistent (Postgres), and vector (Qdrant). Traces record metadata only: token counts, byte sizes, durations, error codes, not prompt text or tool payloads.

Recovery, HITL, and the workflow registry require Postgres when enabled on the server. Graph checkpoint persistence exists; full resume dispatch from graph checkpoints is partial (Graph recovery resume).

## How this doc fits the rest of the site

[Architecture overview](architecture-overview.md) shows how surfaces, the engine, Postgres, and Qdrant connect. [Surfaces and when to use them](surfaces-and-when-to-use-them.md) helps pick SDK vs server vs Relay. [Execution model](execution-model.md) covers linear vs graph and run state machines. [Maturity and known gaps](maturity-and-known-gaps.md) lists what is production-ready vs deferred (server SSE streaming (deferred), operator dashboard UI, OpenTelemetry metrics export, CLI validate command).
