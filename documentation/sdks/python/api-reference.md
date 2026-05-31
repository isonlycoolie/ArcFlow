# Python SDK API reference

**Audience:** `[developer]`

This page documents every symbol in `arcflow.__all__`, plus extension modules searched for LangChain and external outcome helpers. Signatures reflect `sdk-python/arcflow/` as of the current tree.

Import convention:

```python
from arcflow import Agent, Workflow, OpenAI
```

## Workflow

`Workflow(name="default", *, graph=False, runtime=None)`

Declares a linear step list or a graph DAG. Execution happens in Rust via `run()`, `run_stream()`, or remote calls when `runtime=` is set.

| Method | Description |
|--------|-------------|
| `step(agent, *, fallback=None, hitl=None)` | Append a linear step. Raises if `graph=True`. |
| `node(node_id, agent, *, outputs=None)` | Register a graph node. Requires `graph=True`. |
| `add_edge(from_id, to_id=None, *, condition=None)` | Graph edge; `to_id=None` terminates branch. |
| `join_node(join_id, wait_for)` | Join node waiting for listed branch nodes. |
| `set_entry(node_id)` | Override default entry node. |
| `max_iterations(count)` | Graph iteration cap (default 100). |
| `retry(max_attempts, *, backoff=None)` | Step retry policy before `run()`. |
| `timeout(seconds)` | Workflow-level timeout. |
| `step_timeout(seconds)` | Per-step timeout. |
| `enable_recovery(storage="postgresql")` | Enable Postgres-backed recovery. |
| `run(input, *, provider=None, initial_state=None)` | Execute workflow; returns `WorkflowResult`. |
| `run_stream(input, *, provider=None)` | Async iterator of `StreamEvent`. In-process only. |
| `resume(run_id)` | Resume failed linear run when recovery enabled. |
| `resume_with_approval(run_id, approval_key, *, approved=True, data=None)` | HITL resume or server approve when `runtime=` set. |
| `trace()` | Returns `TraceResult` for last run. |
| `test(cases)` | Deterministic stub test cases without LLM. |
| `publish(version, *, published_by=None)` | Publish to server; requires `runtime=`. |
| `resolve(name, version, *, runtime)` | Classmethod; load registry ref into workflow shell. |

`run_stream()` is not supported when `Workflow(..., runtime=...)` points at a remote server.

## Agent

`Agent(name, role, instructions, model="default", tools=(), memory=None, context=None, tool_execution=None)`

Defines a behavioral unit. Does not execute directly.

| Attribute / param | Type | Notes |
|-------------------|------|-------|
| `name`, `role`, `instructions` | `str` | Required non-empty strings |
| `model` | `str` | Declarative; provider model set via `run(provider=...)` |
| `tools` | `tuple[Tool, ...]` | Duplicate tool names rejected at construct time |
| `memory` | `MemoryConfig \| None` | Backend config serialized to Rust |
| `context` | `ContextPolicy \| None` | Prior-step inclusion policy |
| `tool_execution` | `ToolExecutionConfig \| None` | LLM tool loop bounds |
| `agent_id` | `UUID` | Assigned at construction |

## Tool

`Tool(name, description, input_schema, execute, timeout_seconds=30.0)`

| Field | Notes |
|-------|-------|
| `input_schema` | JSON Schema object; validated in Rust |
| `execute` | Callable `(dict) -> str`; invoked by native runtime |
| `timeout_seconds` | Must be positive |

## ContextPolicy

`ContextPolicy(*, include_prior_steps=PriorStepsMode.ALL, include_run_input=True, max_prior_step_chars=4096)`

Controls prompt assembly from prior steps and run input.

## PriorStepsMode

Enum: `ALL`, `LAST`, `NONE`.

## ToolExecutionConfig

`ToolExecutionConfig(*, mode="llm_select", max_iterations=5)`

| `mode` | Meaning |
|--------|---------|
| `llm_select` | Default; LLM chooses tools per iteration |
| `legacy_eager` | Eager tool execution path |

`max_iterations` must be between 1 and 20.

## Memory

### MemoryType

Enum: `SESSION`, `SHARED`, `PERSISTENT`, `VECTOR`.

### MemoryScope

Enum: `AGENT`, `WORKFLOW`, `GLOBAL`.

### MemoryConfig

`MemoryConfig(memory_type, scope=MemoryScope.AGENT, namespace=None, ttl_seconds=None, embedding=None, retrieval=None, chunking=None)`

| Type | Backend | Env var |
|------|---------|---------|
| Session / Shared | In-process | none |
| Persistent | PostgreSQL | `ARCFLOW_POSTGRESQL_URL` |
| Vector | Qdrant | `ARCFLOW_QDRANT_URL` |

`namespace` is required for persistent and vector types.

### MemoryRetrievalConfig

`MemoryRetrievalConfig(mode="dense", dense_weight=0.7, sparse_weight=0.3, rerank=None, top_k=None)`

Hybrid retrieval for vector memory. `rerank` may be `"cohere"` or `"local"`.

### MemoryChunkingConfig

`MemoryChunkingConfig(strategy="recursive", chunk_size=512, overlap=64)`

### VectorStore (submodule, not in `__all__`)

```python
from arcflow.memory import VectorStore, ChunkHit
```

| Method | Returns |
|--------|---------|
| `ingest(namespace, key, text)` | Chunk count (`int`) |
| `search(namespace, query, top_k=5)` | `list[ChunkHit]` |

`ChunkHit` has `text` and `byte_len`.

## Providers

Frozen dataclasses. Credentials from environment only.

| Class | Env var | Constructor |
|-------|---------|-------------|
| `OpenAI` | `OPENAI_API_KEY` | `OpenAI(model, max_tokens=..., temperature=...)` |
| `Anthropic` | `ANTHROPIC_API_KEY` | `Anthropic(model, max_tokens=..., temperature=...)` |
| `Gemini` | `GEMINI_API_KEY` | `Gemini(model, max_tokens=..., temperature=...)` |

Pass to `workflow.run(..., provider=OpenAI(model="gpt-4o"))`.

## WorkflowResult

| Field | Type | Meaning |
|-------|------|---------|
| `output` | `str` | Final step text |
| `run_id` | `str` | UUID for trace and CLI |
| `step_count` | `int` | Steps executed |
| `trace_events` | `tuple[dict, ...]` | Metadata-only RCS events |
| `status` | `str` | Terminal status (e.g. `completed`) |
| `approval_key` | `str \| None` | Set when HITL interrupt occurs |

## Trace types

### TraceResult

| Field | Notes |
|-------|-------|
| `run_id`, `workflow_name`, `status` | Run identity |
| `started_at`, `completed_at` | UTC datetimes |
| `total_duration_seconds`, `total_tokens_consumed` | Aggregates |
| `steps` | `tuple[StepTrace, ...]` |
| `warnings` | e.g. dropped trace events |

Methods: `summary()`, `failed_step()`, iteration support.

### StepTrace

Per-step timing, tokens, tool calls, memory ops, optional `StepError`.

### TokenUsage

`prompt_tokens`, `completion_tokens`, `total_tokens`.

### ToolCallTrace

Metadata only: `tool_name`, `status`, `duration_seconds`, `input_schema_hash`, `output_size_bytes`, `error_code`.

### MemoryOperationTrace

`operation`, `memory_type`, `key`, `hit`, `duration_seconds`.

### StepError

`error_code`, `message`.

