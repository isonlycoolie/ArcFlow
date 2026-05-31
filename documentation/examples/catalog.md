# Examples catalog

**Audience:** `[developer]` `[operator]` `[frontend]`

The `examples/` directory holds runnable samples grouped by concern. Each category below lists what it demonstrates, primary SDK or surface, and whether a README exists in the tree.

Use this catalog to pick a starting point before reading tutorial tracks in capabilities reference §28.

## Index by category

| Directory | Demonstrates | Primary surface | README |
|-----------|--------------|-----------------|--------|
| `examples/static/` | Static site production patterns (Relay + published workflows) | Static SDK, Relay, admin | [README](../../examples/static/README.md) |
| `examples/static/chat-rag/` | Landing-page support chat with RAG | Static SDK, Relay | [README](../../examples/static/chat-rag/README.md) |
| `examples/static/online-application-chatbot/` | Multi-turn intake + external callback | Static SDK, Relay, external | [README](../../examples/static/online-application-chatbot/README.md) |
| `examples/graph/` | DAG routing, joins, parallel fan-out, ReAct-style agents | Python SDK | none (see scripts below) |
| `examples/rag/` | Vector memory, document QA | Python SDK | none |
| `examples/hitl/` | Interrupt and approve flows | Python SDK | none |
| `examples/streaming/` | SDK stream iterators | Python + TypeScript | none |
| `examples/external/` | Webhook callbacks, Playwright stub | Python SDK, server | none |
| `examples/relay/byo-docker/` | Self-hosted Relay | Relay, Docker | [README](../../examples/relay/byo-docker/README.md) |
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
| [chat-rag](../../examples/static/chat-rag/) | Landing-page support chat | ~30 lines: env vars + chat UI |
| [online-application-chatbot](../../examples/static/online-application-chatbot/) | Multi-turn intake with external step | Relay + published workflow ref |

Operator setup flow (from [static README](../../examples/static/README.md)):

1. Dashboard: create site, copy relay URL and site token
2. Dashboard: upload knowledge, configure chat, publish workflow
3. Frontend: set `VITE_ARCFLOW_RELAY_URL` and `VITE_ARCFLOW_SITE_TOKEN`
4. Deploy to CDN; no server-side app code required

Advanced local-only path: `chat-rag/src/main-dev-direct.ts` (direct runtime key; not for production).

## Graph workflows

| Script | Pattern |
|--------|---------|
| [react_agent.py](../../examples/graph/react_agent.py) | ReAct-style tool loop in graph |
| [reflection_loop.py](../../examples/graph/reflection_loop.py) | Conditional reflection routing |
| [parallel_search.py](../../examples/graph/parallel_search.py) | Parallel branches and join |

Requires `Workflow(graph=True)` and Postgres for recovery-heavy paths. Graph resume from checkpoint remains partial (FP-1.01).

## RAG and memory

| Script | Pattern |
|--------|---------|
| [document_qa.py](../../examples/rag/document_qa.py) | Ingest + query with vector memory |

Prerequisites: Qdrant (`ARCFLOW_QDRANT_URL`), embedding provider env vars. Verify `MemoryRetrieved` events in trace.

## Human-in-the-loop

| Script | Pattern |
|--------|---------|
| [expense_approval.py](../../examples/hitl/expense_approval.py) | Step interrupt, approve/reject |

Requires `enable_recovery()` and Postgres. Server path supports approve API when not using in-process SDK only.

## Streaming

| Script | Language |
|--------|----------|
| [chat_stream.py](../../examples/streaming/chat_stream.py) | Python `run_stream()` |
| [chat_stream.ts](../../examples/streaming/chat_stream.ts) | TypeScript `runStream()` |

In-process SDK streaming only. Server SSE is deferred (FP-2).

## External callbacks

| Script | Pattern |
|--------|---------|
| [playwright_stub_callback.py](../../examples/external/playwright_stub_callback.py) | Posts external outcome to server callback |

Pair with `ExternalBindingConfig` on publish and `report_outcome()` from Python. See server HMAC requirements in `contracts/normative/`.

## Relay

| Example | Pattern |
|---------|---------|
| [byo-docker](../../examples/relay/byo-docker/) | Self-hosted Relay with Docker Compose |

