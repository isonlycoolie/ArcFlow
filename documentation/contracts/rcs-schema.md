
# Workflow schema reference reference

Authoritative reference for the ArcFlow workflow specification type system used by workflows, agents, runs, and server API bodies. This page documents the JSON Schema and field semantics.

Validate workflows against schema in CI where possible until `arcflow validate` ships (CLI validate command).

Conceptual intro: [Workflow specification](../concepts/the-rcs-contract.md).

**Drift note (K-10):** Some older HTTP summaries may lag the live server. Prefer this page and the [HTTP API reference](../server/http-api-reference.md) when integrating.

## Core workflow types

### WorkflowDefinition

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | UUID string | Yes | Workflow identity |
| `name` | string | Yes | Human-readable name |
| `execution_mode` | `"linear"` \| `"graph"` | Yes | Execution strategy |
| `steps` | StepDefinition[] | Yes | Step list |
| `graph` | GraphDefinition | When graph mode | DAG definition |
| `external_bindings` | ExternalBinding[] | No | Webhook/async bindings |
| `retry_policy` | RetryPolicy | No | Workflow-level retries |

### StepDefinition

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | UUID string | Yes | Step identity |
| `agent_id` | UUID string | Yes | References AgentDefinition.id |
| `order` | integer | Yes | Linear sort order |
| `fallback_step_id` | UUID | No | Fallback step on failure |
| `hitl` | HitlConfig | No | Human-in-the-loop gate |
| `retry_policy` | RetryPolicy | No | Step-level retries |

### AgentDefinition

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | UUID string | Yes | Agent identity |
| `name` | string | Yes | Display name |
| `role` | string | Yes | Role label in traces |
| `instructions` | string | Yes | System instructions (not in traces, trace data policy) |
| `tools` | ToolDefinition[] | No | Registered tools |
| `memory_config` | MemoryConfig | No | Memory backend config |
| `context` | ContextPolicy | No | Prior step context limits |
| `tool_execution` | ToolExecutionConfig | No | Default `llm_select` |
| `provider` | ProviderConfig | No | LLM provider selection |

UUID ids throughout; do not reuse ids across agents and steps.

## Graph types

### GraphDefinition

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `entry_node` | string | Yes | First graph node id |
| `max_iterations` | integer | Yes | Loop guard |
| `nodes` | GraphNode[] | Yes | Node list |
| `edges` | GraphEdge[] | Yes | Transitions |
| `join_nodes` | JoinNode[] | No | Parallel branch sync |

### GraphNode

| Field | Type | Notes |
|-------|------|-------|
| `id` | string | Node id |
| `step_ref` | string | References StepDefinition.id |
| `inputs` | object | Optional input mapping |
| `outputs` | object | Optional output mapping |

### GraphEdge

| Field | Type | Notes |
|-------|------|-------|
| `from` | string | Source node id |
| `to` | string or null | Target node; null ends branch |
| `condition` | string | Optional guard expression |

### JoinNode

| Field | Type | Notes |
|-------|------|-------|
| `id` | string | Join id |
| `wait_for` | string[] | Node ids to synchronize |

Graph trace kinds in the workflow specification envelope: `GraphNodeStarted`, `GraphNodeCompleted`, `GraphIterationLimitReached`.

## Memory and context

### MemoryConfig

| Field | Type | Notes |
|-------|------|-------|
| `memory_type` | enum | e.g. vector, session |
| `scope` | enum | run vs persistent |
| `namespace` | string | Required for persistent/vector |
| `ttl_seconds` | integer | Optional TTL |
| `embedding` | object | Embedding model config |
| `retrieval` | MemoryRetrievalConfig | Search mode |
| `chunking` | MemoryChunkingConfig | Ingest chunking |

### MemoryRetrievalConfig

| Field | Type | Notes |
|-------|------|-------|
| `mode` | string | dense, sparse, hybrid |
| `top_k` | integer | Result count |
| `dense_weight` | float | Hybrid weight |
| `sparse_weight` | float | Hybrid weight |
| `rerank` | object | Optional rerank provider |

### MemoryChunkingConfig

| Field | Type | Notes |
|-------|------|-------|
| `chunk_size` | integer | Characters or tokens per policy |
| `chunk_overlap` | integer | Overlap between chunks |

### ContextPolicy

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `include_prior_steps` | bool | | Prior step outputs in context |
| `include_run_input` | bool | | Original user input |
| `max_prior_step_chars` | integer | 4096 | Truncation limit |

### ToolExecutionConfig

| Field | Type | Notes |
|-------|------|-------|
| `mode` | `"llm_select"` \| `"legacy_eager"` | Tool loop strategy |
| `max_iterations` | integer | Tool loop cap |

## Control plane types

### HitlConfig

| Field | Type | Notes |
|-------|------|-------|
| `approval_key` | string | Approve route key |
| `timeout_seconds` | integer | Interrupt timeout |

### ExternalBinding

| Field | Type | Notes |
|-------|------|-------|
| `id` | string | Binding id for external POST |
| `kind` | string | Binding type |
| `attach_to_step_id` | UUID | Step attachment |
| `mode` | string | async/monitor semantics |
| `recovery` | object | Recovery policy on external failure |

### ProviderConfig

| Field | Type | Notes |
|-------|------|-------|
| `provider_id` | string | e.g. openai |
| `model` | string | Model id |
| `api_key_env` | string | Env var name for key |
| `params` | object | Temperature, max_tokens |

### ToolDefinition

| Field | Type | Notes |
|-------|------|-------|
| `name` | string | Tool name in traces |
| `input_schema` | JSON Schema | Arguments schema |
| `permissions` | string[] | Optional capability flags |

## Run envelope

### RunRequest (server POST /v1/runs)

| Field | Type | Notes |
|-------|------|-------|
| `workflow` | WorkflowDefinition | Inline definition |
| `workflow_ref` | object | `{ name, version }` semver range |
| `agents` | AgentDefinition[] | Agent catalog |
| `input` | string | User/run input |
| `exec_config` | ExecConfig | Recovery, timeouts |

Provide `workflow` **or** `workflow_ref`, not both.

### RunResult

| Field | Type | Notes |
|-------|------|-------|
| `output` | string | Final text |
| `status` | ExecutionStatus | Terminal or in-flight |
| `trace` | TraceEvent[] | metadata-only trace events |
| `error` | object | Error code + message |

### ExecutionStatus

`Pending`, `Running`, `Completed`, `Failed`, `Retrying`, `Cancelled`, `Interrupted`.

### ErrorCode

See [Error codes](error-codes.md).

## Example minimal linear workflow

```json
{
 "id": "00000000-0000-4000-8000-000000000001",
 "name": "demo",
 "execution_mode": "linear",
 "steps": [
 {
 "id": "00000000-0000-4000-8000-000000000010",
 "agent_id": "00000000-0000-4000-8000-000000000020",
 "order": 0
 }
 ]
}
```

## Normative file index

| Path | Content |
|------|---------|
| [workflow schema](../contracts/rcs-schema.md) | JSON Schema |
| [Postgres schema](../server/postgres-schema.md) | Recovery DDL (partial) |
| [Provider configuration](../guides/agents-and-tools/provider-configuration.md) | Provider API |
| [workflow schema](../contracts/rcs-schema.md) | Index |

## Related pages

- [Error codes](error-codes.md)
- [Trace events normative](trace-events-normative.md)
- [Graph workflows](../guides/workflows/graph-workflows.md)
