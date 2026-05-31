**Audience:** `[developer]`

# Context policies

`ContextPolicy` controls what prior information an agent sees on each step: earlier step outputs, the original run input, and size limits. It applies in both [Linear workflows](../workflows/linear-workflows.md) and [Graph workflows](../workflows/graph-workflows.md). Graph state from node `outputs` may also arrive via `exec_config.initial_state` and the runtime map.

Agent overview: [Defining agents](defining-agents.md). RCS types: [The RCS contract](../../concepts/the-rcs-contract.md).

## ContextPolicy fields

```json
{
  "context": {
    "include_prior_steps": "last",
    "include_run_input": true,
    "max_prior_step_chars": 4096
  }
}
```

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `include_prior_steps` | `"none"` \| `"last"` \| `"all"` | `"last"` | Which prior step outputs to inject |
| `include_run_input` | boolean | `true` | Include original run input string |
| `max_prior_step_chars` | integer | `4096` | Truncate prior step text to this length |

Engine default for `max_prior_step_chars` is 4096 (verified in `types.rs`).

## include_prior_steps modes

### `"last"` (default)

Only the immediately preceding step output is included. Typical for pipelines where each stage refines the previous result.

```json
{
  "context": {
    "include_prior_steps": "last",
    "include_run_input": true,
    "max_prior_step_chars": 4096
  }
}
```

### `"all"`

All completed prior steps are included up to `max_prior_step_chars` budget (implementation applies per-step truncation rules). Use when downstream agents need full chain context; watch token costs.

### `"none"`

Prior step text is omitted. The agent still receives run input if `include_run_input` is true. Useful for isolated classification steps that should not be biased by earlier prose.

```json
{
  "context": {
    "include_prior_steps": "none",
    "include_run_input": true,
    "max_prior_step_chars": 4096
  }
}
```

## include_run_input

When `true`, the original `input` passed to `workflow.run(input)` or `POST /v1/runs` is available to the agent prompt assembly. Set `false` when later steps should only see structured prior outputs or memory, not the raw user message.

Example for a summarize-only second step:

```json
{
  "context": {
    "include_prior_steps": "last",
    "include_run_input": false,
    "max_prior_step_chars": 8192
  }
}
```

## max_prior_step_chars

Prevents unbounded prompt growth from large intermediate outputs. Prior text is trimmed before provider request (`ProviderRequestSent` records `prompt_size_bytes` in trace, not content).

Raise the limit for long-document workflows:

```json
{
  "context": {
    "include_prior_steps": "all",
    "include_run_input": true,
    "max_prior_step_chars": 16384
  }
}
```

Balance against model context window and cost.

## Graph workflows and initial_state

Graph nodes may write keys via `outputs`:

```json
{
  "id": "n-classify",
  "step_ref": "s-classify",
  "outputs": { "route": "category" }
}
```

Seed or resume graph state through exec_config:

```json
{
  "exec_config": {
    "initial_state": {
      "route": "billing",
      "observation": "Customer mentioned invoice 8842"
    }
  }
}
```

Context policy and graph state compose: policy controls step output text; graph state supplies structured fields the engine merges into agent context per scheduler rules.

## Memory vs context policy

| Mechanism | Purpose |
|-----------|---------|
| Context policy | Ephemeral prompt assembly from run history |
| [Memory types](../memory-and-rag/memory-types.md) | Durable or scoped key-value and vector retrieval |

Use shared memory for multi-agent handoff that must survive truncation limits:

```json
{
  "memory_config": {
    "memory_type": "shared",
    "scope": "workflow",
    "namespace": "pipeline-state"
  }
}
```

## Multi-agent linear example

Step 1 researcher with full user input; step 2 writer with last step only:

```json
{
  "agents": [
    {
      "id": "a-research",
      "name": "researcher",
      "role": "Research",
      "instructions": "Research the topic.",
      "context": {
        "include_prior_steps": "none",
        "include_run_input": true,
        "max_prior_step_chars": 4096
      }
    },
    {
      "id": "a-write",
      "name": "writer",
      "role": "Writer",
      "instructions": "Write a summary of the research.",
      "context": {
        "include_prior_steps": "last",
        "include_run_input": false,
        "max_prior_step_chars": 8192
      }
    }
  ]
}
```

## Trace visibility

Context assembly is not a separate trace event. Related events:

- `AgentInvoked` with `input_size_bytes`
- `ProviderRequestSent` with `prompt_size_bytes`

No prompt text appears in traces (SEC-1).

## Related pages

- [Defining agents](defining-agents.md)
- [Linear workflows](../workflows/linear-workflows.md)
- [Graph workflows](../workflows/graph-workflows.md)
- [SEC-1 and data safety](../../concepts/sec-1-and-data-safety.md)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §5.1; Appendix A (ContextPolicy); K-15 default 4096 chars.
