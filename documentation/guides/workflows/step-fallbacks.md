**Audience:** `[developer]`

# Step fallbacks

Step fallbacks route a failed primary step to a backup step instead of failing the entire workflow immediately. You set `fallback_step_id` on a `StepDefinition` to point at another step id in the same workflow. When the primary agent fails in a way that triggers fallback, the engine activates the fallback step and emits `StepFallbackActivated` in the trace.

Fallbacks complement [Retry and backoff](../reliability/retry-and-backoff.md): retry handles transient provider errors; fallback handles "try a cheaper model" or "escalate to a specialist agent" patterns after primary failure or alongside retry exhaustion.

## When to use fallbacks

| Scenario | Primary | Fallback |
|----------|---------|----------|
| Model tier degradation | GPT-4 class agent | Faster mini model |
| Specialist escalation | General support agent | Domain expert agent |
| Tool-heavy vs text-only | Agent with tools | Text-only summarizer |

Fallback is not a substitute for [Graph workflows](graph-workflows.md) routing. It activates on step failure, not on conditional output.

## RCS configuration

```json
{
  "id": "00000000-0000-4000-8000-000000000001",
  "name": "resilient_qa",
  "execution_mode": "linear",
  "steps": [
    {
      "id": "00000000-0000-4000-8000-000000000010",
      "agent_id": "00000000-0000-4000-8000-000000000020",
      "order": 1,
      "fallback_step_id": "00000000-0000-4000-8000-000000000012"
    },
    {
      "id": "00000000-0000-4000-8000-000000000011",
      "agent_id": "00000000-0000-4000-8000-000000000021",
      "order": 2,
      "fallback_step_id": null
    },
    {
      "id": "00000000-0000-4000-8000-000000000012",
      "agent_id": "00000000-0000-4000-8000-000000000022",
      "order": 3,
      "fallback_step_id": null
    }
  ]
}
```

In this layout, step `010` primary agent `020` falls back to step `012` (agent `022`) on qualifying failure. Step `011` has no fallback. The fallback step's `order` value still participates in linear sort when the fallback path is not taken; when fallback activates, the engine runs the fallback step's agent in place of terminal failure.

Both primary and fallback agents must be defined in the run's `agents` array.

## Agent definitions for primary and fallback

```json
[
  {
    "id": "00000000-0000-4000-8000-000000000020",
    "name": "primary_analyst",
    "role": "Senior analyst",
    "instructions": "Answer with full tool access.",
    "provider": {
      "provider_id": "openai",
      "model": "gpt-4o",
      "api_key_env": "OPENAI_API_KEY"
    }
  },
  {
    "id": "00000000-0000-4000-8000-000000000022",
    "name": "fallback_analyst",
    "role": "Backup analyst",
    "instructions": "Answer concisely without tools.",
    "provider": {
      "provider_id": "openai",
      "model": "gpt-4o-mini",
      "api_key_env": "OPENAI_API_KEY"
    }
  }
]
```

See [Defining agents](../agents-and-tools/defining-agents.md) and [Provider configuration](../agents-and-tools/provider-configuration.md).

## Trace sequence

When fallback activates:

```json
[
  { "kind": "StepStarted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000010", "agent_name": "primary_analyst" },
  { "kind": "StepFailed", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000010", "error_code": "ProviderError" },
  { "kind": "StepFallbackActivated", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000010", "primary_agent_name": "primary_analyst", "fallback_agent_name": "fallback_analyst" },
  { "kind": "StepStarted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000012", "agent_name": "fallback_analyst" },
  { "kind": "StepCompleted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000012", "duration_ms": 650 }
]
