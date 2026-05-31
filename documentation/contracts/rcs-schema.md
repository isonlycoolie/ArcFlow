**Audience:** `[developer]` `[platform]`

# RCS schema reference

Authoritative reference for the Runtime Contract Specification (RCS) type system used by workflows, agents, runs, and server API bodies. JSON Schema: [contracts/normative/rcs/v1.schema.json](../../contracts/normative/rcs/v1.schema.json). Rust types: `runtime/arcflow-core/src/rcs/types.rs`.

Validate workflows against schema in CI where possible until `arcflow validate` ships (FP-5.04).

Conceptual intro: [The RCS contract](../concepts/the-rcs-contract.md).

**Drift note (K-10):** [contracts/normative/runtime/server-api-v1.md](../../contracts/normative/runtime/server-api-v1.md) is partially stale. Prefer this page, Appendix A in the capabilities reference, and live server handlers for HTTP shapes.

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
| `instructions` | string | Yes | System instructions (not in traces, SEC-1) |
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

Graph trace kinds in RCS envelope: `GraphNodeStarted`, `GraphNodeCompleted`, `GraphIterationLimitReached`.

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
