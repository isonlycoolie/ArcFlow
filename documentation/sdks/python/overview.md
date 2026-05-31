# Python SDK overview

**Audience:** `[developer]`

The ArcFlow Python SDK lets you declare multi-agent workflows in Python and execute them in the Rust runtime (`arcflow-core`). Python owns structure: agents, steps, tools, memory config, graph topology, and run options. The native extension owns execution: LLM calls, tool validation, memory I/O, tracing, recovery, and streaming.

This split keeps workflow definitions readable in application code while preserving deterministic, SEC-1-safe execution in Rust.

## What you can build

| Capability | Python surface | Notes |
|------------|----------------|-------|
| Linear workflows | `Workflow().step(agent)` | Default path; stub agent when no provider |
| Graph workflows | `Workflow(graph=True)` with `node()`, `add_edge()`, `join_node()` | Conditional routing, joins, parallel branches |
| Tools | `Tool` on `Agent(tools=...)` | JSON Schema validated in Rust; Python supplies callables |
| Memory | `MemoryConfig`, `VectorStore` | Session, shared, Postgres persistent, Qdrant vector |
| LLM providers | `OpenAI`, `Anthropic`, `Gemini` | Keys from environment only |
| Recovery | `enable_recovery()` | Postgres-backed; linear resume supported; graph resume partial (FP-1.01) |
| HITL | `HitlConfig` on `step()` | Interrupt, approve/reject via `resume_with_approval()` |
| Streaming | `run_stream()` | In-process only; not supported with remote `runtime=` URL |
| External callbacks | `ExternalBindingConfig`, `report_outcome()` | HMAC-signed POST to server |
| Observability | `WorkflowResult.trace_events`, `workflow.trace()` | Metadata only; no raw prompts or tool payloads |
| Registry | `Workflow(..., runtime=...)`, `publish()`, `resolve()` | Server-backed workflow refs |
| Schedules | `ScheduleManifest` | Validates `arcflow.schedule.yaml` structure |
| LangChain interop | `arcflow.langchain` submodule | Optional; not in top-level `__all__` |

## Architecture

```
Your Python app
    |
    v
arcflow (PyO3 extension + thin Python wrappers)
    |
    v
arcflow-core (Rust)
    |
    +-- LLM providers (OpenAI, Anthropic, Gemini)
    +-- Tool execution + jsonschema validation
    +-- Memory backends (in-process, Postgres, Qdrant)
    +-- Trace store (SEC-1 metadata events)
    +-- Recovery / HITL / external resume
```

Python has no runtime dependencies beyond the built extension. Development installs use maturin to compile the native module from this repository.

## Typical workflow

```python
from arcflow import Agent, OpenAI, Workflow

researcher = Agent(
    name="researcher",
    role="research",
    instructions="Research the topic and list key facts.",
)
writer = Agent(
    name="writer",
    role="write",
    instructions="Write a concise summary from the research.",
)

wf = Workflow("research_pipeline")
wf.step(researcher)
wf.step(writer)

result = wf.run(
    "Analyze renewable energy trends",
    provider=OpenAI(model="gpt-4o"),
)
print(result.output)
print(result.run_id)

trace = wf.trace()
print(trace.summary())
```

Without `provider=`, `run()` uses the stub agent. That path needs no API keys and is useful for structure tests and CI.

## Public API boundary

Top-level imports come from `arcflow/__init__.py` via `__all__`. Additional helpers live in submodules:

| Module | Purpose |
|--------|---------|
| `arcflow.langchain` | LangChain tool and LangGraph conversion (optional extra) |
| `arcflow.memory` | `VectorStore`, `ChunkHit` (not re-exported at package root) |

Names like `FromLangChain`, `LangChainToArcflow`, and `CommonTools` do not exist in this codebase. The LangChain adapter exports `from_langchain_tool`, `to_arcflow_step`, `langgraph_to_arcflow`, and `langgraph_to_rcs_json` from `arcflow.langchain`.

## Parity and gaps

Python is the reference SDK surface. TypeScript matches core workflow, graph, recovery, HITL, streaming, and vector ingest, but its `Agent` class does not yet expose tools, memory, or context policy in the TypeScript binding layer. See [parity matrix](../parity-matrix.md) for a full cross-surface comparison.

Known runtime gaps that affect both SDKs:

| Gap | Impact on Python SDK |
|-----|----------------------|
| FP-1.01 Graph recovery resume | `resume()` works for linear runs; mid-graph resume incomplete |
| FP-2 Server SSE | Use `run_stream()` in-process or poll server GET run |
| Remote runtime + streaming | `run_stream()` raises `WorkflowConfigurationError` when `runtime=` is set |

## Related pages

| Page | Content |
|------|---------|
| [Installation](installation.md) | maturin, pip, platform notes |
| [API reference](api-reference.md) | All `__all__` exports and extension modules |
| [Exception reference](exception-reference.md) | Error hierarchy and remediation |
| [Python quickstart](../../getting-started/quickstart-python.md) | First run with traces |
| [Parity matrix](../parity-matrix.md) | Python vs TypeScript vs server |

## Source

`sdk-python/arcflow/__init__.py`, `sdk-python/README.md`, `sdk-python/arcflow/workflow.py`; capabilities reference §16, §16.1, §16.2.
