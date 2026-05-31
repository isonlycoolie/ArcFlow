**Audience:** `[developer]` `[platform]`

# The RCS contract

RCS (Runtime Contract Schema) v1 is the shared language between ArcFlow surfaces and `arcflow-core`. Workflows, agents, run requests, trace envelopes, and error codes are defined as JSON with a normative schema before Python, TypeScript, or HTTP handlers add their own ergonomics. Contract-first design is why a graph workflow validated in the CLI matches behavior on the server and in Relay-backed browser runs.

## Why contract-first matters

Without a single schema, each surface would drift: the server might accept a field the Python SDK never sends, or traces might omit fields the VS Code timeline expects. ArcFlow treats [contracts/normative/rcs/v1.schema.json](../../contracts/normative/rcs/v1.schema.json) and Rust types in `runtime/arcflow-core/src/rcs/types.rs` as the source of truth for structure. Surface APIs are adapters, not parallel definitions.

The hierarchy in practice:

1. **Normative contracts** under `contracts/normative/` when marked current and matching code
2. **Capabilities reference** for end-to-end flows, parity, and JSON examples
3. **Source code** when contracts are silent or known stale

Server route documentation note: [contracts/normative/runtime/server-api-v1.md](../../contracts/normative/runtime/server-api-v1.md) may lag the implemented routes in `server/arcflow-server/src/lib.rs`. Prefer the capabilities reference Appendix B or [server/](../server/overview.md) docs when integrating via HTTP.

## Core RCS types

### WorkflowDefinition

| Field | Role |
|-------|------|
| `id`, `name` | Stable identity and trace labels |
| `execution_mode` | `"linear"` or `"graph"` |
| `steps` | Ordered step list; each references an agent by UUID |
| `graph` | Required for graph mode: nodes, edges, join_nodes |
| `external_bindings` | Optional HTTP callbacks attached to steps |
| `retry_policy` | Optional workflow-level retry |

### StepDefinition

Each step has `id`, `agent_id`, `order` (linear sort key), optional `fallback_step_id`, optional `hitl`, optional `retry_policy`.

### AgentDefinition

Agents carry `instructions`, optional `tools` with JSON Schema inputs, `memory_config`, `context` policy, `tool_execution` mode (`llm_select` default), and `provider` with `api_key_env` pointing at an environment variable name, not a secret value.

### Run envelope

| Type | Purpose |
|------|---------|
| `RunRequest` | Workflow or `workflow_ref`, agents, input, `exec_config` |
| `RunResult` | Output, status, trace reference, error |
| `ExecutionStatus` | `Pending`, `Running`, `Completed`, `Failed`, `Retrying`, `Cancelled`, `Interrupted` |
| `ErrorCode` | Typed failure reasons shared across SDK and HTTP |

Example minimal linear workflow fragment:

```json
{
  "id": "00000000-0000-4000-8000-000000000001",
  "name": "research_pipeline",
  "execution_mode": "linear",
  "steps": [
    { "id": "00000000-0000-4000-8000-000000000010", "agent_id": "00000000-0000-4000-8000-000000000020", "order": 1 },
    { "id": "00000000-0000-4000-8000-000000000011", "agent_id": "00000000-0000-4000-8000-000000000021", "order": 2 }
  ]
}
```

## SDKs as interface layers

Python and TypeScript do not redefine workflow semantics. They expose native types that serialize to the same RCS JSON the engine consumes.

**Python** (`sdk-python`): `Workflow`, `Agent`, `workflow.run(input, recovery_enabled=True)`. Errors map to `WorkflowConfigurationError`, `WorkflowExecutionError`, `InfrastructureUnavailableError` from RCS codes.

**TypeScript** (`sdk-typescript`): `Workflow`, `Agent`, `workflow.run(input, { recoveryEnabled: true })`. Used by VS Code and Node tests.

**HTTP** (`arcflow-server`): Request and response DTOs mirror `RunRequest` / run status JSON. Registry routes store and resolve semver workflow versions.

**Static browser** (`@arcflow/static`): Does not embed full workflow JSON in production. Calls `runPublished("chat", "^1.0.0", message)` so definitions live in the server registry after admin publish.

**CLI**: Executes local workflow files; migrate applies SQL migrations aligned with server schema. Full `arcflow validate` against schema is deferred (FP-5.04).

The engine entry for all paths is `WorkflowEngine::execute_with_config`. Validation runs via `validate_workflow` and `validate_graph` before execution.

## exec_config bridge

`exec_config` is part of the run contract, not the workflow file alone. Common fields:

```json
{
  "recovery_enabled": true,
  "workflow_timeout_secs": 600,
  "step_timeout_secs": 120,
  "stream": { "enabled": false },
  "initial_state": {},
  "retry": { "max_attempts": 3, "backoff": { "kind": "exponential", "base_ms": 1000 } }
}
```

Recovery gates Postgres persistence. Test mode (`exec_config.test`) allows deterministic CI with per-step `output`, `fail_times`, and `then_output` without live LLM calls.

## Observability contract

Trace event names and allowed fields are normative under [contracts/normative/observability/trace-events-v1.md](../../contracts/normative/observability/trace-events-v1.md). SEC-1 requires metadata-only payloads (see [SEC-1 and data safety](sec-1-and-data-safety.md)).

## Validation today

Rust validates before run. CI should validate workflow JSON against `v1.schema.json` until `arcflow validate` ships (FP-5.04). Graph recovery resume uses checkpoint schema in Postgres but dispatch is incomplete (FP-1.01).

## Related pages

- [Execution model](execution-model.md) for linear vs graph scheduling
- [Architecture overview](architecture-overview.md) for where RCS crosses HTTP boundaries
- [contracts/rcs-schema.md](../contracts/rcs-schema.md) for narrative schema guide

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §2 (Design principles), §4 (Workflow execution), §22 (Contracts and schemas), Appendix A (RCS type reference).
