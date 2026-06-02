# Examples catalog


The `examples/` directory holds runnable samples grouped by concern. Each category below lists what it demonstrates, primary SDK or surface, and whether a README exists in the tree.

Use this catalog to pick a starting point before reading [tutorial tracks](../tutorials/track-a-first-workflow.md).

## Index by category

| Directory | Demonstrates | Primary surface | README |
|-----------|--------------|-----------------|--------|
| `examples/static/` | Static site production patterns (Relay + published workflows) | Static SDK, Relay, admin | [README](../examples/static-chat-widget.md) |
| `examples/static/chat-rag/` | Landing-page support chat with RAG | Static SDK, Relay | [Static chat widget walkthrough](../examples/static-chat-widget.md) |
| `examples/static/online-application-chatbot/` | Multi-turn intake + external callback | Static SDK, Relay, external | [README](../examples/static-chat-widget.md) |
| `examples/graph/` | DAG routing, joins, parallel fan-out, ReAct-style agents | Python SDK | none (see scripts below) |
| `examples/rag/` | Vector memory, document QA | Python SDK | none |
| `examples/hitl/` | Interrupt and approve flows | Python SDK | none |
| `examples/streaming/` | SDK stream iterators | Python + TypeScript | none |
| `examples/external/` | Webhook callbacks, Playwright stub | Python SDK, server | none |
| `examples/relay/byo-docker/` | Self-hosted Relay | Relay, Docker | [README](../examples/relay-byo-deployment.md) |
| `examples/langchain/` | LangChain / LangGraph interop | Python SDK (`arcflow.langchain`) | none |
| `examples/education/` | Domain vertical: course QA | Python SDK | none |
| `examples/healthcare/` | Domain vertical: protocol QA | Python SDK | none |
| `examples/personal/` | Domain vertical: blog pipeline | Python SDK | none |
| `examples/support/` | Domain vertical: ticket RAG bot | Python SDK | none |
| `examples/trading/` | Domain vertical: research RAG trade | Python SDK | none |

## Static product examples

Production static-site path: dashboard owns agents, memory, and knowledge; frontend calls `runPublished()` through Relay.

| Example | Use case | Frontend responsibility |
|---------|----------|-------------------------|
| [Static chat widget](../examples/static-chat-widget.md) | Landing-page support chat | ~30 lines: env vars + chat UI |
| [Static chat widget](../examples/static-chat-widget.md) | Multi-turn intake with external step | Relay + published workflow ref |

Operator setup flow (from [static README](../examples/static-chat-widget.md)):

1. Dashboard: create site, copy relay URL and site token
2. Dashboard: upload knowledge, configure chat, publish workflow
3. Frontend: set `VITE_ARCFLOW_RELAY_URL` and `VITE_ARCFLOW_SITE_TOKEN`
4. Deploy to CDN; no server-side app code required

Advanced local-only path: `chat-rag/src/main-dev-direct.ts` (direct runtime key; not for production).

## Graph workflows

| Script | Pattern |
|--------|---------|
| [react_agent.py](../examples/graph-routing.md) | ReAct-style tool loop in graph |
| [reflection_loop.py](../examples/graph-routing.md) | Conditional reflection routing |
| [parallel_search.py](../examples/graph-routing.md) | Parallel branches and join |

Requires `Workflow(graph=True)` and Postgres for recovery-heavy paths. Graph resume from checkpoint remains partial (Graph recovery resume).

## RAG and memory

| Script | Pattern |
|--------|---------|
| [document_qa.py](../examples/rag-chatbot.md) | Ingest + query with vector memory |

Prerequisites: Qdrant (`ARCFLOW_QDRANT_URL`), embedding provider env vars. Verify `MemoryRetrieved` events in trace.

## Human-in-the-loop

| Script | Pattern |
|--------|---------|
| [expense_approval.py](../examples/hitl-approval-flow.md) | Step interrupt, approve/reject |

Requires `enable_recovery()` and Postgres. Server path supports approve API when not using in-process SDK only.

## Streaming

| Script | Language |
|--------|----------|
| [chat_stream.py](../examples/streaming-responses.md) | Python `run_stream()` |
| [chat_stream.ts](../examples/streaming-responses.md) | TypeScript `runStream()` |

In-process SDK streaming only. Server SSE is deferred (streaming deferred).

## External callbacks

| Script | Pattern |
|--------|---------|
| [playwright_stub_callback.py](../examples/external-webhook.md) | Posts external outcome to server callback |

Pair with `ExternalBindingConfig` on publish and `report_outcome()` from Python. See server HMAC requirements in published contract pages.

## Relay

| Example | Pattern |
|---------|---------|
| [Relay BYO deployment](../examples/relay-byo-deployment.md) | Self-hosted Relay with Docker Compose |

Browser env: `VITE_ARCFLOW_RELAY_URL`, `VITE_ARCFLOW_SITE_TOKEN`. Relay validates Origin and rate-limits per site.

## LangChain interop

| Script | Pattern |
|--------|---------|
| [migration_demo.py](../examples/catalog.md) | `from_langchain_tool`, `langgraph_to_arcflow` |

Requires Python optional `[langchain]` extra. No TypeScript equivalent in repo.

## Vertical samples

Domain-specific agent workflows (Python). Useful as copy sources, not separate product surfaces:

| Directory | Focus |
|-----------|-------|
| `education/` | Course Q&A |
| `healthcare/` | Protocol Q&A |
| `personal/` | Blog content pipeline |
| `support/` | Ticket RAG bot |
| `trading/` | Research + RAG trade flow |

Each is a single-script sample without a dedicated README. Run from repo root with SDK built and provider keys set.

## VS Code extension examples

| Path | Content |
|------|---------|
| [extensions/vscode-arcflow/examples/](../vscode/overview.md) | Workflow preview JSON for extension dev |

Separate from top-level `examples/`; targets extension authors.

## How to run (general)

1. Build the SDK for your language ([Python install](../sdks/python/installation.md) or [TypeScript install](../sdks/typescript/installation.md))
2. Export provider and backend env vars the sample needs
3. Run the script from repository root, e.g. `python examples/rag/document_qa.py`

Static examples use `npm install` and `npm run dev` inside the example directory.

## Gaps in example coverage

| Topic | Status |
|-------|--------|
| TypeScript graph sample in `examples/graph/` | Python only today |
| TypeScript RAG sample | Python only; TS has `VectorStore` but no dedicated example README |
| Server-only curl tutorials | See [Track B: Server API](../tutorials/track-b-server-api.md), not duplicated here |
| Per-example README | Only static and relay categories have README files today |

Contributors adding examples should include a README with prerequisites, env vars, and a verify command per [Example catalog](../examples/catalog.md).

## Branch note (pending merges)

Documentation and lessons may reference names from branches not yet on `main`:

| Current branch path | After `feat/examples-catalog-restructure` |
|---------------------|---------------------------------------------|
| `examples/rag/document_qa.py` | `examples/rag/memory_guide_qa.py` (ingest + query in one script) |
| `examples/personal/blog_pipeline.py` | `examples/personal/weekly_blog_pipeline.py` |
| `examples/external/playwright_stub_callback.py` | `examples/external/portal_outcome_callback.py` |
| `examples/langchain/migration_demo.py` | `examples/langchain/langchain_adapter_demo.py` |

Lessons under [getting-started/rag/](../getting-started/rag/README.md) call out both names where relevant. After merge, update this table and remove dual references.

SDK PascalCase facades (`CommonTools`, `FromLangChain`, `ExternalOutcome`) are documented in [getting-started/tools/](../getting-started/tools/README.md) with legacy import notes until `feat/sdk-pascalcase-facades` merges.
