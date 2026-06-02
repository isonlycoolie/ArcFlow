# Track A: First workflow


Track A is the first structured tutorial in the ArcFlow learning path. You build a linear two-agent pipeline, run it with the stub provider (no API keys), and verify completion status plus trace lifecycle events. The same workflow can be written in Python or TypeScript; pick one language for this track.

## Goal

Complete a linear two-agent pipeline using the stub provider. Confirm `result.status == "completed"` (SDK) and that trace metadata includes `WorkflowStarted`, `StepCompleted`, and `WorkflowCompleted`.

## Prerequisites

| Item | Required |
|------|----------|
| SDK installed | [Install and build](../getting-started/install-and-build.md) |
| Curriculum (recommended) | [Fundamentals](../getting-started/fundamentals/README.md), [Writing workflows](../getting-started/writing-workflows/01-linear-pipelines.md) |
| API keys | Not required (stub default) |
| Docker / Postgres | Not required for embedded SDK |

Optional reading: [First workflow in five minutes](../getting-started/first-workflow-in-five-minutes.md) for the minimal script without verification steps.

## Primary example

The canonical pipeline matches the five-minute guide: a researcher step followed by a writer step on the topic "Analyze renewable energy trends".

## Step 1: Create the workflow file

### Python (`track_a.py`)

```python
from arcflow import Agent, Workflow

researcher = Agent(
 name="researcher",
 role="research",
 instructions="Research the given topic thoroughly.",
)
writer = Agent(
 name="writer",
 role="write",
 instructions="Write a clear summary of the research.",
)

workflow = Workflow("track-a")
workflow.step(researcher)
workflow.step(writer)

result = workflow.run("Analyze renewable energy trends")

print("output:", result.output[:120], "..." if len(result.output) > 120 else "")
print("run_id:", result.run_id)
print("step_count:", result.step_count)
print("status:", result.status)
```

### TypeScript (`track_a.ts`)

```typescript
import { Agent, Workflow } from "arcflow";

const researcher = new Agent({
 name: "researcher",
 role: "research",
 instructions: "Research the given topic thoroughly.",
});

const writer = new Agent({
 name: "writer",
 role: "write",
 instructions: "Write a clear summary of the research.",
});

const workflow = new Workflow({ name: "track-a" });
workflow.step(researcher);
workflow.step(writer);

const result = await workflow.run("Analyze renewable energy trends");

const preview =
 result.output.length > 120 ? `${result.output.slice(0, 120)}...` : result.output;
console.log("output:", preview);
console.log("run_id:", result.runId);
console.log("step_count:", result.stepCount);
console.log("status:", result.status);
```

Run:

```bash
# Python
python track_a.py

# TypeScript (from a project with arcflow installed)
node --experimental-strip-types track_a.ts
# or compile with tsc and node dist/track_a.js
```

## Step 2: Verify run outcome

Check these conditions before moving on:

| Check | Expected |
|-------|----------|
| `result.output` (or `result.output` in TS) | Non-empty string |
| Step count | `2` |
| `run_id` / `runId` | UUID string |
| `status` | `completed` (SDK lowercase) |

Python one-liner assertion (append to your script):

```python
assert result.step_count == 2
assert result.status == "completed"
assert len(result.output) > 0
assert result.run_id
print("track A run checks passed")
```

TypeScript equivalent:

```typescript
if (result.stepCount !== 2) throw new Error("expected 2 steps");
if (result.status !== "completed") throw new Error(`unexpected status: ${result.status}`);
if (!result.output) throw new Error("empty output");
if (!result.runId) throw new Error("missing runId");
console.log("track A run checks passed");
```

These assertions mirror `sdk-python/tests/integration/test_first_five_minutes.py`.

## Step 3: Verify trace lifecycle events

After `run()`, inspect raw trace events on the result:

### Python

```python
kinds = {event.get("event_kind") for event in result.trace_events}
required = {"WorkflowStarted", "StepCompleted", "WorkflowCompleted"}
missing = required - kinds
if missing:
 raise SystemExit(f"missing trace kinds: {missing}")
print("trace kinds ok:", sorted(kinds))
```

### TypeScript

```typescript
const events = (result.traceEvents ?? []) as Array<{ event_kind?: string }>;
const kinds = new Set(events.map((e) => e.event_kind).filter(Boolean));
const required = ["WorkflowStarted", "StepCompleted", "WorkflowCompleted"] as const;
for (const kind of required) {
 if (!kinds.has(kind)) throw new Error(`missing trace kind: ${kind}`);
}
console.log("trace kinds ok:", [...kinds].sort());
```

Event field name is `event_kind` in SDK exports (see `test_memory_workflow.py` and trace bridge). Payloads are metadata only per trace data policy.

## Step 4: Use structured `trace()`

### Python

```python
trace = workflow.trace()
assert trace.run_id == result.run_id
assert len(trace) == 2
assert trace.status in ("completed", "partial")
print(trace.summary())
```

### TypeScript

```typescript
const trace = workflow.trace();
if (trace.runId !== result.runId) throw new Error("runId mismatch");
if (trace.steps.length !== 2) throw new Error("expected 2 step traces");
console.log(trace.summary());
```

`partial` may appear when a step completes with degraded metadata; for this stub pipeline, `completed` is typical.

## Step 5: Optional CLI trace view

From the repository root, with the same Python process still holding the in-process trace store, or after any local SDK run:

```bash
cargo run -p arcflow-cli -- trace YOUR_RUN_ID --format json --verbose
```

Replace `YOUR_RUN_ID` with the UUID printed by your script. This is optional for Track A but previews operator tooling used in Track H.

## Expected output (summary)

You should see:

1. Printed output text from the stub pipeline (content varies by version).
2. `step_count=2`, `status=completed`, and a UUID `run_id`.
3. Trace kind set containing at least `WorkflowStarted`, `StepCompleted`, and `WorkflowCompleted`.
4. `trace.summary()` reporting two steps and zero or stub token counts.

Exact stub strings are not part of the pass criteria; the integration tests assert structure, not LLM prose.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `ImportError: arcflow` | SDK not built | `maturin develop` in `sdk-python/` or `npm run build` in `sdk-typescript/` |
| `TraceNotFoundError` on `trace()` | `trace()` before `run()` | Call `run()` first on the same `Workflow` instance |
| Empty `trace_events` | Very old binding | Rebuild native extension |
| `WorkflowConfigurationError` | Invalid agent or empty name | Match the example field names exactly |

## What you learned

Track A establishes the core loop: define agents, register ordered steps, call `run()`, read `output`, and confirm observability through trace events. Later tracks add server HTTP (Track B), vector memory (Track C), graph routing (Track D), and human-in-the-loop flows (Track E).

## Next tracks

| Track | Focus | When to start |
|-------|-------|---------------|
| B | Server API, Docker, `POST /v1/runs` | After A or in parallel if you are a platform engineer |
| C | RAG and Qdrant | After A; requires Docker dev stack |
| H | CLI and VS Code | After A; local tooling |

Continue with [Python quickstart](../getting-started/quickstart-python.md) or [TypeScript quickstart](../getting-started/quickstart-typescript.md) when you want a real LLM provider.
