<p align="center">
  <img height="100" alt="ArcFlow" src="assets/brand/arcflow.png">
</p>

<p align="center">
  <b>Production workflow runtime for AI agent pipelines</b>
</p>

<p align="center">
  <a href="https://github.com/isonlycoolie/ArcFlow/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/isonlycoolie/ArcFlow/ci.yml?style=flat-square" alt="CI status"></a>
  <a href="contracts/README.md"><img src="https://img.shields.io/badge/Contracts-RCS%20%2B%20API-blue?style=flat-square" alt="Contracts"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-green?style=flat-square" alt="License"></a>
</p>

<p align="center">
  <strong>
    <a href="#what-is-arcflow">Overview</a> •
    <a href="#why-arcflow">Why ArcFlow</a> •
    <a href="#capabilities">Capabilities</a> •
    <a href="#deployment-modes">Deployment</a> •
    <a href="#sdks--tools">SDKs</a> •
    <a href="#examples">Examples</a> •
    <a href="#contracts--api">Contracts</a> •
    <a href="#quick-start">Quick Start</a>
  </strong>
</p>

## What is ArcFlow

**ArcFlow** is a self-hosted AI workflow runtime built in Rust. It executes multi-step agent pipelines: LLM calls, structured tool loops, vector memory, branching logic, and human approval gates, with the same semantics whether you embed it in Python, call it from TypeScript, or run it behind an HTTP server.

You define workflows as ordered steps or directed graphs. The runtime owns orchestration: step scheduling, retries, timeouts, state propagation, recovery checkpoints, and trace correlation. Language SDKs are thin bindings over a single engine, not parallel reimplementations of the same logic.

ArcFlow targets teams that need **production-grade agent infrastructure on their own stack**: predictable behavior under load, explicit failure modes, privacy-safe observability, and wire formats you can version and audit.

## Why ArcFlow

Most agent frameworks optimize for getting a demo running quickly. ArcFlow optimizes for **running the same workflow in production for months**: identical execution across languages, typed errors instead of silent degradation, and contracts that outlive any one SDK release.

### One engine, every surface

Orchestration lives entirely in `arcflow-core` (Rust). Python and TypeScript serialize workflow definitions into the Runtime Contract Specification (RCS), invoke the engine, and deserialize results. A retry policy fix or recovery bug patch ships once and applies everywhere. See [ADR-001](docs/architecture/ADR-001.md).

### Contract-first integration

RCS is a versioned JSON schema for workflows, agents, messages, and trace events. If two components disagree on behavior, the schema is the arbiter. Normative specs under [contracts/](contracts/README.md) define server APIs, provider boundaries, recovery DDL, and observability rules so operators can integrate without reading Rust internals.

### Self-hosted by design

You run the binary, Postgres, and vector store. API keys stay in your environment. Traces record metadata (step timing, token counts, status codes), not prompt or completion text (SEC-1). No mandatory cloud control plane.

### Built for failure, not just happy paths

Recovery persists run state to Postgres and resumes from the failed step after restart. Per-step and workflow-level retries use configurable backoff. Human-in-the-loop gates block execution until an operator approves, with durable approval state. Missing infrastructure (e.g. Postgres when recovery is enabled) returns a typed error instead of falling back silently.

## Capabilities

### Workflow execution

- **Linear pipelines**, ordered steps with deterministic scheduling and context handoff between agents
- **Graph workflows (DAG)**, branch, join, and conditional routing for multi-path agent logic
- **Execution modes**, explicit configuration for embedded in-process runs vs remote server execution
- **Workflow registry**, semver `workflow_ref` resolution so published definitions can evolve without breaking callers

### Agents, tools, and providers

- **Multi-agent steps**, each step binds an agent with role, instructions, and optional tool attachments
- **Structured tool loops**, OpenAI-compatible function calling with schema-validated inputs
- **Provider boundary**, swap OpenAI, Anthropic, Gemini, or custom providers at run time without rewriting workflow structure
- **Stub and live paths**, deterministic stubs for CI and local dev; real providers for production runs

### Memory and knowledge

- **Session, shared, and persistent memory**, scoped reads and writes coordinated by the runtime
- **Vector memory (RAG)**, Qdrant-backed retrieval for knowledge-grounded agents
- **Dashboard-driven knowledge**, production static sites ingest and publish knowledge server-side; the browser never embeds documents or embedding keys

### Reliability and control

- **Postgres-backed recovery**, checkpoint after each step; `resume(run_id)` continues from `failed_at + 1`
- **Retry and timeout policies**, per-step and workflow-level limits with exponential backoff
- **Human-in-the-loop**, approval gates that pause execution until an operator acts, with persistent gate state
- **Intelligent retry triggers**, retry on classified transient failures without re-running completed steps

### Observability and operations

- **Metadata-only execution traces**, every run emits structured events: step started, LLM invoked, tool executed, memory read/write, failures, retries
- **OpenTelemetry export**, opt-in OTLP spans and metrics for Grafana, Jaeger, or Prometheus ([observability guide](docker/observability-otel.md))
- **CLI and VS Code extension**, validate workflows, inspect traces, step through runs locally ([vscode-arcflow](extensions/vscode-arcflow/README.md))

### Static and edge delivery

- **ArcFlow Relay**, origin-validated proxy for static frontends; site tokens and workflow definitions stay off the CDN bundle
- **`runPublished()`**, browser clients invoke semver-pinned published workflows by name, no agent definitions in client code
- **Edge WASM (alpha)**, stub linear workflows on Cloudflare Workers for low-latency edge paths while the full edge story matures

## Architecture

ArcFlow stacks three layers. SDKs and the HTTP server are adapters; the engine and contract sit below.

```text
┌─────────────────────────────────────────────────────────┐
│  SDKs (Python, TypeScript) · CLI · arcflow-server HTTP  │
├─────────────────────────────────────────────────────────┤
│  RCS, versioned JSON contract (workflows, traces)       │
├─────────────────────────────────────────────────────────┤
│  arcflow-core, engine, agents, tools, memory, recovery │
└─────────────────────────────────────────────────────────┘
```

Fault tolerance, validation, and scheduling are implemented once in the engine layer. SDKs pass `ExecutionConfig` (retry, timeout, recovery flags) as JSON; they do not reimplement orchestration.

## Deployment modes

Pick the runtime surface that matches where your workflow runs.

### Self-hosted server

The full engine: graph workflows, Postgres recovery, vector memory, real LLM providers, workflow registry, and the public HTTP API. For backend services, long-running pipelines, and operator-controlled infrastructure.

**Docs:** [server/arcflow-server/README.md](server/arcflow-server/README.md)

### ArcFlow Relay

A secure bridge for static sites (Vite, Next.js export, CDN). The dashboard publishes workflows and holds secrets; the browser calls relay with a site token only. For production chat and intake flows without API keys in the frontend bundle.

**Docs:** [examples/static/README.md](examples/static/README.md)

### Edge WASM (alpha)

Stub linear workflows compiled to WASM on Cloudflare Workers. Low latency at the CDN edge without a central round-trip. For lightweight paths while edge parity with the full server matures.

**Docs:** [docker/edge-deployment-cloudflare.md](docker/edge-deployment-cloudflare.md)

## SDKs & Tools

- **[sdk-python](sdk-python/README.md)**, Python workflow definitions backed by the Rust runtime; full recovery, retry, and provider support
- **[sdk-typescript](sdk-typescript/README.md)**, Promise-native TypeScript SDK with N-API bindings to the same engine
- **[packages/arcflow-static](packages/arcflow-static)**, Browser client for relay mode (`runPublished`)
- **[extensions/vscode-arcflow](extensions/vscode-arcflow/README.md)**, Workflow graph, trace timeline, local step-through debug
- **[cli/arcflow-cli](cli/arcflow-cli)**, `arcflow` CLI: validate, run, trace, and TUI

## Examples

- **[examples/static/chat-rag](examples/static/chat-rag/)**, Landing-page support chat with RAG via relay
- **[examples/static/online-application-chatbot](examples/static/online-application-chatbot/)**, Multi-turn intake with external callback
- **[examples/relay/byo-docker](examples/relay/byo-docker/)**, Self-hosted relay with the same browser contract as managed relay

## Contracts & API

Production wire formats and operator guides live under [contracts/](contracts/README.md):

- **[RCS v1 schema](contracts/normative/rcs/v1.schema.json)**, Workflow and message data model
- **[Server API v1](contracts/normative/runtime/server-api-v1.md)**, Self-hosted HTTP API
- **[Trace events v1](contracts/normative/observability/trace-events-v1.md)**, Observability event schema and SEC-1 rules
- **[Provider API v1](contracts/normative/providers/api-v1.md)**, LLM provider boundary

Validate the RCS schema: `bash scripts/validate-rcs-schema.sh`

## Quick Start

For a first run, start dev dependencies (`docker compose -f docker/docker-compose.dev.yml up -d`), then follow [server/arcflow-server/README.md](server/arcflow-server/README.md) or the [Python SDK guide](sdk-python/README.md). Static-site production paths start at [examples/static/README.md](examples/static/README.md).

## Contributing

Contributions are welcome. Before opening a pull request:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Keep commits focused. Local check for commit size: `bash scripts/check-commit-size.sh --commit HEAD`.

## License

ArcFlow is licensed under **MIT OR Apache-2.0** at your option. See [Cargo.toml](Cargo.toml) for workspace metadata.
