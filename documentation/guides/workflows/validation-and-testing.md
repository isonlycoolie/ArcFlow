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

Stub is for tests only. [First workflow in five minutes](../../getting-started/first-workflow-in-five-minutes.md) uses stub by default.

## Graph validation checklist

For [Graph workflows](graph-workflows.md):

1. Every `step_ref` in `nodes` matches a step `id`
2. `entry_node` exists in `nodes`
3. All `edges.from` and `edges.to` reference valid node ids (or `to: null`)
4. `join_nodes[].wait_for` lists nodes reachable on parallel paths
5. `max_iterations` is set to a sane bound

Test routing with test mode outputs that match edge conditions:

```json
{
  "test": {
    "steps": {
      "s-classify": { "output": "billing" },
      "s-billing": { "output": "billing resolution" },
      "s-merge": { "output": "merged summary" }
    }
  }
}
```

## Server test run

```bash
curl -s -X POST http://localhost:8080/v1/runs \
  -H "Authorization: Bearer $ARCFLOW_SERVER_API_KEY" \
  -H "Content-Type: application/json" \
  -d @test-run.json
```

Poll `GET /v1/runs/{run_id}/trace` and assert event kinds. Traces are SEC-1 safe; see [SEC-1 and data safety](../../concepts/sec-1-and-data-safety.md).

## Recovery testing

Linear recovery with test mode:

```json
{
  "recovery_enabled": true,
  "test": {
    "steps": {
      "00000000-0000-4000-8000-000000000010": { "output": "step one" },
      "00000000-0000-4000-8000-000000000011": { "fail_times": 999, "output": "never" }
    }
  }
}
```

Simulate interrupt mid-run, then resume per [Recovery and resume](../reliability/recovery-and-resume.md). **Graph checkpoint resume is partial (FP-1.01)**; do not treat graph resume tests as release gates until FP-1.01 closes.

## Verification matrix

| Goal | Mechanism | Expected trace |
|------|-----------|----------------|
| Happy path | `output` only | `WorkflowCompleted` |
| Retry | `fail_times: 1`, retry config | `RetryAttempted` |
| Fallback | primary fail + `fallback_step_id` | `StepFallbackActivated` |
| Invalid graph | bad `step_ref` | `WorkflowValidationFailed` |
| Timeout | low `step_timeout_secs` | `TimeoutEnforced` |

## Related pages

- [Linear workflows](linear-workflows.md)
- [Graph workflows](graph-workflows.md)
- [Maturity and known gaps](../../concepts/maturity-and-known-gaps.md) (FP-5.04, FP-1.01)
- [Install and build](../../getting-started/install-and-build.md)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §4.5, Appendix C (test mode); FP-5.04 CLI validate stub; §27 known gaps.
