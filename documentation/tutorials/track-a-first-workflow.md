# Track A: First workflow

**Audience:** `[developer]`

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
