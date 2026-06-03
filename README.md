<p align="center">
  <img height="100" alt="ArcFlow" src="documentation/assets/brand/arcflow.png">
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
    <a href="documentation/README.md">Documentation</a> •
    <a href="#contracts--api">Contracts</a> •
    <a href="#quick-start">Quick Start</a>
  </strong>
</p>

## What is ArcFlow

**ArcFlow** is a self-hosted AI workflow runtime built in Rust. It executes multi-step agent pipelines: LLM calls, structured tool loops, vector memory, branching logic, and human approval gates, with the same semantics whether you embed it in Python, call it from TypeScript, or run it behind an HTTP server.

You define workflows as ordered steps or directed graphs. The runtime owns orchestration: step scheduling, retries, timeouts, state propagation, recovery checkpoints, and trace correlation. Language SDKs are thin bindings over a single engine, not parallel reimplementations of the same logic.

> **One engine, every surface.** Orchestration lives in `arcflow-core` (Rust). SDKs serialize workflow definitions into the Runtime Contract Specification (RCS), invoke the engine, and deserialize results. A fix in retry policy or recovery ships once and applies everywhere.

ArcFlow targets teams that need **production-grade agent infrastructure on their own stack**: predictable behavior under load, explicit failure modes, privacy-safe observability, and wire formats you can version and audit.

## Why ArcFlow

Most agent frameworks optimize for getting a demo running quickly. ArcFlow optimizes for **running the same workflow in production for months**: identical execution across languages, typed errors instead of silent degradation, and contracts that outlive any one SDK release.

<table>
<tr>
<td width="50%" valign="top">

### One engine, every surface

Orchestration lives entirely in `arcflow-core` (Rust). Python and TypeScript serialize workflow definitions into RCS, invoke the engine, and deserialize results. See [Architecture overview](documentation/concepts/architecture-overview.md).

</td>
<td width="50%" valign="top">

### Contract-first integration

RCS is a versioned JSON schema for workflows, agents, messages, and trace events. Normative specs under [contracts/](contracts/README.md) define provider boundaries, recovery DDL, and observability rules.

</td>
</tr>
<tr>
<td width="50%" valign="top">

### Self-hosted by design

You run the binary, Postgres, and vector store. API keys stay in your environment. Traces record metadata (step timing, token counts, status codes), not prompt or completion text. No mandatory cloud control plane.

</td>
<td width="50%" valign="top">

### Built for failure

Recovery persists run state to Postgres and resumes from the failed step after restart. Per-step retries use configurable backoff. Human-in-the-loop gates block until an operator acts. Missing infrastructure returns a typed error instead of falling back silently.

</td>
</tr>
</table>

## Capabilities

<table>
<tr><th>Area</th><th>What you get</th></tr>
<tr>
<td><strong>Workflow execution</strong></td>
<td>
<strong>Linear pipelines</strong> with deterministic scheduling and context handoff<br>
<strong>Graph workflows (DAG)</strong> with branch, join, and conditional routing<br>
<strong>Execution modes</strong> for embedded in-process vs remote server runs<br>
<strong>Workflow registry</strong> with semver <code>workflow_ref</code> resolution
</td>
</tr>
<tr>
<td><strong>Agents, tools, providers</strong></td>
<td>
<strong>Multi-agent steps</strong> with role, instructions, and optional tools<br>
<strong>Structured tool loops</strong> with schema-validated OpenAI-compatible function calling<br>
<strong>Provider boundary</strong>, swap OpenAI, Anthropic, Gemini at run time<br>
<strong>Stub and live paths</strong> for CI and production
</td>
</tr>
<tr>
<td><strong>Memory and knowledge</strong></td>
<td>
<strong>Session, shared, and persistent memory</strong> scoped by the runtime<br>
<strong>Vector memory (RAG)</strong> with Qdrant-backed retrieval<br>
<strong>Dashboard-driven knowledge</strong>, documents and embedding keys stay server-side
</td>
</tr>
<tr>
<td><strong>Reliability and control</strong></td>
<td>
<strong>Postgres-backed recovery</strong>, <code>resume(run_id)</code> from failed step<br>
<strong>Retry and timeout policies</strong> per step and workflow<br>
<strong>Human-in-the-loop</strong> with durable approval state<br>
<strong>Intelligent retry triggers</strong> on classified transient failures
</td>
</tr>
<tr>
<td><strong>Observability</strong></td>
<td>
<strong>Metadata-only execution traces</strong>, step timing, tokens, tool/memory events<br>
<strong>OpenTelemetry export</strong> for Grafana, Jaeger, Prometheus (<a href="docker/observability-otel.md">guide</a>)<br>
<strong>CLI and VS Code extension</strong> for validate, trace, step-through debug
</td>
</tr>
<tr>
<td><strong>Static and edge</strong></td>
<td>
<strong>ArcFlow Relay</strong>, origin-validated proxy; secrets off the CDN bundle<br>
<strong><code>runPublished()</code></strong>, semver-pinned workflows from the browser<br>
<strong>Edge WASM (alpha)</strong>, stub linear workflows on Cloudflare Workers
</td>
</tr>
</table>

## Architecture

ArcFlow's power comes from concentrating orchestration in a single, feature-rich engine while exposing that behavior through multiple language surfaces. The engine in `runtime/arcflow-core` implements the core primitives you need to run production AI workflows:

- Agents: first-class agent identities and execution contexts that encapsulate prompts, tools, and provider bindings.
- Workflows: ordered pipelines and directed graphs (DAGs) with conditional routing, joins, and re-entry semantics.
- Graph execution: native support for graphs (not just linear chains), with graph-state coordination, checkpoint persistence, and resume semantics.
- RAG / Vector memory: embedding, vector store retrieval, and memory coordination lives adjacent to the engine so retrieval-augmented generation integrates cleanly into step execution.
- Tools & loops: structured tool-calling and deterministic tool loops that produce typed outputs and integrate with retry/compensation logic.
- Human-in-the-loop (HITL) and external callbacks: durable approval state, HMAC-verified callbacks, and external outcome routing.

Adapters (the SDKs, CLI, server, and edge proxies) are intentionally thin: they serialize workflow definitions and `ExecutionConfig` into the Runtime Contract Specification (RCS), call the engine, and surface typed results. This makes the engine the single place where scheduling, retries, timeouts, state commits, trace emission, and recovery are implemented — a fix in core behavior fixes all SDKs at once.

Multi-language surfaces: ArcFlow supports and maintains SDKs across languages (Python, TypeScript), and is actively extended to additional languages (Java, Go), plus a CLI and VS Code integration for local development and debugging. Language SDKs do not reimplement orchestration — they translate developer intent into RCS and forward it to the engine.

Persistence and observability:
- Recovery and checkpoints: Postgres-backed recovery state and graph checkpoint upsert (see `contracts/normative/runtime/recovery-schema-v2.sql` and `runtime/arcflow-core/src/workflow/graph/checkpoint.rs`). Linear resume and structured replay are implemented via `runtime/arcflow-core/src/recovery/resume.rs` and `runtime/arcflow-core/src/workflow/run.rs`.
- Trace events and metrics: metadata-first execution traces are emitted from the engine and can be exported via OpenTelemetry for metrics, tracing, and monitoring.

Deployment surfaces:
- Self-hosted server (`server/arcflow-server`) provides admin APIs, run endpoints, and trace persistence for long-running workloads.
- ArcFlow Relay and `arcflow-static` act as edge/proxy surfaces that validate tokens and either proxy to a full engine or run constrained stub workflows for low-latency paths.
- Edge WASM is experimental for stubbed linear workflows where full graph and recovery features are not required.

Implementation notes and current gaps: graph checkpoint persistence exists and is invoked from the scheduler (`runtime/arcflow-core/src/workflow/graph/checkpoint.rs`, scheduler call sites in `runtime/arcflow-core/src/workflow/graph/scheduler.rs`). Resume for sorted/linear runs is implemented via the recovery module (`runtime/arcflow-core/src/recovery/resume.rs` → `run_sorted_steps`). The HTTP server currently exposes POST and GET run endpoints (`server/arcflow-server/src/handlers/runs.rs`); a streaming SSE `/v1/runs/{id}/events` endpoint is not implemented and SDKs either stream in-process or poll the server for updates.

This arrangement — a single, authoritative engine with multi-language adapters and versioned contracts — is what makes ArcFlow suitable for long-lived, auditable, and recoverable AI workflows in production.

## Deployment modes
Choose the deployment mode that fits your needs. Below are the common modes and when to prefer each.

### Self-hosted server
Use this when you need graph workflows, Postgres-backed recovery, vector memory, a workflow registry, or an HTTP API for backend services. See the arcflow-server docs for setup and operational notes: [arcflow-server](server/arcflow-server/README.md).

### ArcFlow Relay
Use Relay for static sites (Vite, Next.js exports, CDN). Relay validates site tokens and proxies requests so LLM API keys never ship in browser bundles. See the static examples for deployment patterns: [Static examples](examples/static/README.md).

### Edge WASM (alpha)
Edge WASM is an experimental mode for low-latency, linear-stub workflows at the CDN edge. It is not feature-complete, graph routing, RAG, and recovery are not supported. See the Cloudflare guide for an example: [Cloudflare guide](docker/edge-deployment-cloudflare.md).

## SDKs & Tools

ArcFlow exposes the same runtime through language SDKs, CLI tooling, and editor integrations. Pick the surface that matches your development and deployment needs.

- **Python SDK:** Workflow definitions and runtime bindings for Python; use for scripts, services, and notebooks. See [sdk-python](sdk-python/README.md).
- **TypeScript SDK:** N-API backed bindings for Node and tooling; powers VS Code integration and Node services. See [sdk-typescript](sdk-typescript/README.md).
- **Browser client (arcflow-static):** Client for published workflows running via Relay; use `runPublished()` in production browser flows. See [packages/arcflow-static](packages/arcflow-static).
- **VS Code extension:** Graph visualization, local runs, and trace timelines for authoring and debugging. See [extensions/vscode-arcflow](extensions/vscode-arcflow/README.md).
- **CLI:** Local `arcflow` commands for `run`, `trace`, `migrate`, and lightweight validation during development. See [cli/arcflow-cli](cli/arcflow-cli).

## Examples

A few curated examples to explore:

- **Static chat with RAG:** Landing-page support chat that uses Relay and RAG, see [examples/static/chat-rag](examples/static/chat-rag/).
- **Online application chatbot:** Multi-turn intake with external callbacks, see [examples/static/online-application-chatbot](examples/static/online-application-chatbot/).
- **Self-hosted Relay (BYO):** Example self-hosted Relay deployment, see [examples/relay/byo-docker](examples/relay/byo-docker/).

## Contracts & API

Normative wire formats, schemas, and integrator-facing routes live under the `contracts` and `documentation` folders. Key references:

- **RCS v1 schema:** Workflow and message data model, [contracts/normative/rcs/v1.schema.json](contracts/normative/rcs/v1.schema.json)
- **HTTP API reference:** Server routes and admin surfaces, [documentation/server/http-api-reference.md](documentation/server/http-api-reference.md)
- **Trace events v1:** Observability event schema and trace data policy, [contracts/normative/observability/trace-events-v1.md](contracts/normative/observability/trace-events-v1.md)
- **Provider API v1:** Provider-facing boundary for LLMs, [contracts/normative/providers/api-v1.md](contracts/normative/providers/api-v1.md)

Validate the RCS schema with `bash scripts/validate-rcs-schema.sh`.

## Quick Start

For a first run, start dev dependencies (`docker compose -f docker/docker-compose.dev.yml up -d`), then follow [server/arcflow-server/README.md](server/arcflow-server/README.md) or the [Python SDK guide](sdk-python/README.md). Static-site production paths start at [examples/static/README.md](examples/static/README.md).

## Contributing

Contributions are welcome.

<details>
<summary>Pre-push checks</summary>

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Keep commits focused. Local check for commit size: `bash scripts/check-commit-size.sh --commit HEAD`.

</details>

## License

ArcFlow is licensed under **MIT OR Apache-2.0** at your option. See [Cargo.toml](Cargo.toml) for workspace metadata.
