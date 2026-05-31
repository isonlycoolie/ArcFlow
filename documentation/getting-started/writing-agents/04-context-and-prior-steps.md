# 04 Context and prior steps

**Audience:** `[developer]`

## Before you start

Complete [03 Roles and multi-agent pipelines](03-roles-and-multi-agent-pipelines.md). You should have a two-step pipeline running with the stub provider and understand that step order determines execution sequence.

## Concept

When the runtime builds the prompt for an agent, it can include:

| Source | Controlled by |
|--------|---------------|
| The original string passed to `workflow.run(input)` | `include_run_input` |
| Text output from earlier steps | `include_prior_steps` |
| Maximum length of prior step text | `max_prior_step_chars` |

These three fields form a `ContextPolicy`. Pass it to `Agent(..., context=ContextPolicy(...))`.

### include_prior_steps

| Value | Behavior |
|-------|----------|
| `"last"` | Only the immediately preceding step output (common default for pipelines) |
| `"all"` | All completed prior step outputs, subject to the char limit |
| `"none"` | No prior step text; useful for isolated classification steps |

### include_run_input

When `True`, the agent still sees the original run input. When `False`, later steps rely on prior step output only. A writer that should summarize research but ignore the raw user message often sets `include_run_input=False`.

### max_prior_step_chars

Caps how much prior text enters the prompt (minimum 256 in the Python SDK). Prevents runaway prompt size when intermediate steps produce long output.

Default behavior when you omit `context` is defined by the runtime. Explicit policies make multi-agent handoffs predictable before you connect a paid provider.

## Example

Researcher sees the user topic but no prior steps (it runs first). Writer sees the researcher's output but not the raw run input.

Save as `context_policy_demo.py`:

```python
from arcflow import Agent, ContextPolicy, Workflow

researcher = Agent(
    name="researcher",
    role="Research",
    instructions="Research the given topic thoroughly.",
    context=ContextPolicy(
        include_prior_steps="none",
        include_run_input=True,
        max_prior_step_chars=4096,
    ),
)

writer = Agent(
    name="writer",
    role="Writer",
    instructions="Write a summary of the research only.",
    context=ContextPolicy(
        include_prior_steps="last",
        include_run_input=False,
        max_prior_step_chars=8192,
    ),
)

workflow = Workflow("context-demo")
workflow.step(researcher)
workflow.step(writer)

result = workflow.run("Analyze renewable energy trends")

print(result.output[:200], "..." if len(result.output) > 200 else "")
print(f"status={result.status} steps={result.step_count}")
```

Run:

```bash
python context_policy_demo.py
```

The researcher policy keeps the first step focused on the user input without empty prior-step noise. The writer policy forces it to work from the researcher's output rather than re-reading the original topic verbatim.

## Verify

| Check | Expected |
|-------|----------|
| Script completes | No configuration error |
| `result.status` | `"completed"` |
| `result.step_count` | `2` |

