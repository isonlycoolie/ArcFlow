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
    <a href="#getting-started">Quick Start</a> •
    <a href="#contracts--api">Contracts</a> •
    <a href="#sdks--tools">SDKs</a> •
    <a href="#examples">Examples</a> •
    <a href="#deployment-modes">Deployment</a> •
    <a href="#contributing">Contributing</a>
  </strong>
</p>

**ArcFlow** is a Rust workflow runtime for AI agent pipelines. It combines a deterministic execution engine, versioned RCS contracts, and SDKs for Python, TypeScript, and the browser. Define ordered or graph workflows, run them locally or on a self-hosted server, and ship production chat experiences through ArcFlow Relay — without embedding agents or API keys in static frontends.

ArcFlow is written in Rust for reliability under load. Recovery, vector memory, observability, and provider boundaries are first-class — not bolted on after the fact.

## Getting Started

### Dev dependencies

Start Postgres and Qdrant for local development:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

### Run the server

From the repo root:

```bash
export ARCFLOW_SERVER_API_KEY=dev-secret
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow

docker compose -f docker/docker-compose.server.yml up --build
```

See [server/arcflow-server/README.md](server/arcflow-server/README.md) for migrations, SDK remote mode, and load testing.

### Your first workflow

```python
from arcflow import Agent, Workflow

wf = Workflow()
wf.step(Agent(name="writer", role="author", instructions="Write a concise summary."))
result = wf.run("Explain vector databases in two sentences")
print(result.output)
```

TypeScript and remote-server usage: [sdk-python/README.md](sdk-python/README.md) · [sdk-typescript/README.md](sdk-typescript/README.md)

## Deployment modes

| Mode | Best for | Docs |
|------|----------|------|
| **Self-hosted server** | Full engine — graph workflows, Postgres recovery, vector memory, real LLM providers | [server/arcflow-server/README.md](server/arcflow-server/README.md) |
| **ArcFlow Relay** | Static sites (Vite, Next.js export, CDN) — dashboard publishes workflows; browser calls relay only | [examples/static/README.md](examples/static/README.md) |
| **Edge WASM (alpha)** | Stub linear workflows on Cloudflare Workers — low latency, no central round-trip | [docker/edge-deployment-cloudflare.md](docker/edge-deployment-cloudflare.md) |

## SDKs & Tools

| Component | Purpose |
|-----------|---------|
| [sdk-python](sdk-python/README.md) | Python workflow definitions backed by the Rust runtime |
| [sdk-typescript](sdk-typescript/README.md) | Promise-native TypeScript SDK |
| [packages/arcflow-static](packages/arcflow-static) | Browser client for relay mode (`runPublished`) |
| [extensions/vscode-arcflow](extensions/vscode-arcflow/README.md) | Workflow graph, trace timeline, local step-through debug |
| [cli/arcflow-cli](cli/arcflow-cli) | `arcflow` CLI — validate, run, trace, and TUI |

## Examples

| Example | Use case |
|---------|----------|
| [examples/static/chat-rag](examples/static/chat-rag/) | Landing-page support chat with RAG via relay |
| [examples/static/online-application-chatbot](examples/static/online-application-chatbot/) | Multi-turn intake with external callback |
| [examples/relay/byo-docker](examples/relay/byo-docker/) | Self-hosted relay with the same browser contract as managed relay |

## Features

- **Linear and graph workflows** — RCS-defined agents, steps, and execution modes
- **Postgres-backed recovery** — resume runs after failure or restart
- **Workflow registry** — semver `workflow_ref` resolution on the server
- **Vector memory** — Qdrant-backed RAG for knowledge-grounded agents
- **OpenTelemetry-native tracing** — opt-in OTLP export for spans and metrics ([observability guide](docker/observability-otel.md))
- **Human-in-the-loop** — approval gates with persistent state
- **Provider boundary** — OpenAI-compatible LLM integration with structured tool loops

## Contracts & API

Production wire formats and operator guides live under [contracts/](contracts/README.md):

| Document | Purpose |
|----------|---------|
| [RCS v1 schema](contracts/normative/rcs/v1.schema.json) | Workflow and message data model |
| [Server API v1](contracts/normative/runtime/server-api-v1.md) | Self-hosted HTTP API |
| [Trace events v1](contracts/normative/observability/trace-events-v1.md) | Observability event schema |
| [Provider API v1](contracts/normative/providers/api-v1.md) | LLM provider boundary |

Validate the RCS schema:

```bash
bash scripts/validate-rcs-schema.sh
```

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
