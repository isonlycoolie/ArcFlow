**Audience:** `[developer]` `[platform]`

# Validation and testing

ArcFlow validates workflow definitions before execution and supports deterministic test mode for CI without live LLM calls. Validation catches schema errors, broken graph references, and missing agent ids early. Test mode simulates step outputs and controlled failures so you can verify retry, fallback, and trace shape cheaply.

Normative schema: [contracts/normative/rcs/v1.schema.json](../../../contracts/normative/rcs/v1.schema.json). Rust types: `runtime/arcflow-core/src/rcs/types.rs`. Conceptual overview: [The RCS contract](../../concepts/the-rcs-contract.md).

## Engine validation

Before a run starts, the engine calls:

| Function | Scope |
|----------|-------|
| `validate_workflow` | Steps, agent references, mode consistency |
| `validate_graph` | Nodes, edges, join_nodes, step_ref integrity (graph mode) |

Failure surfaces as `WorkflowValidationFailed` in trace and `InvalidWorkflowDefinition` error code (HTTP 400 on server).

Common failures:

- Step `agent_id` not present in run agents list
- Graph `step_ref` pointing at missing step id
- Join node `wait_for` referencing unknown node ids
- `execution_mode: "graph"` without `graph` object

## CLI validate (FP-5.04)

```bash
arcflow validate workflow.json
```

**Status: stub.** Full schema validation in the CLI is deferred (**FP-5.04**, plan ref `feat/fp-5-cli-validate`). Today:

- Use JSON Schema validation in CI against `v1.schema.json`
- Rely on engine validation at run time
- Use `arcflow run` with test mode for integration checks

Do not document `arcflow validate` as production-complete until FP-5.04 ships.

## JSON Schema in CI

Example with a generic validator (adapt to your toolchain):

```bash
npx ajv validate -s contracts/normative/rcs/v1.schema.json -d my-workflow.json
```

Validate both workflow and agent bundles before `PUT` registry publish. See [Workflow registry](workflow-registry.md).

## Test mode (exec_config.test)

`ExecutionConfig.test` drives per-step mock behavior without calling providers.

```json
{
  "recovery_enabled": false,
  "test": {
    "steps": {
      "00000000-0000-4000-8000-000000000010": {
        "output": "mock step 1",
        "fail_times": 0
      },
      "00000000-0000-4000-8000-000000000011": {
        "fail_times": 1,
        "output": "fail first",
        "then_output": "success second attempt"
      }
    }
  }
}
```

| Field | Behavior |
|-------|----------|
| `output` | Text returned when step succeeds |
| `fail_times` | Number of attempts that fail before success |
| `then_output` | Output after failures exhausted or on subsequent attempt per engine rules |

Use `fail_times` to exercise [Retry and backoff](../reliability/retry-and-backoff.md) traces (`RetryAttempted`, `RetryExhausted`). Combine with [Step fallbacks](step-fallbacks.md) to assert `StepFallbackActivated`.

## Stub provider

For runs that still invoke the agent loop but need deterministic LLM behavior, set agent provider to stub:

```json
{
  "provider": {
    "provider_id": "stub",
    "model": "stub-v1",
    "api_key_env": ""
  }
}
```

