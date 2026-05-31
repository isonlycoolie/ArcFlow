# TypeScript quickstart


## Before you start

The TypeScript SDK exposes the same Rust execution engine as Python with Promise-based APIs and Node.js idioms. If you want the smallest example first, see [First workflow in five minutes](first-workflow-in-five-minutes.md) (Python) or jump to the example below.

Requires Node.js 18+. For install issues, see [Install and build](install-and-build.md).

## Concept

Behavior matches Python for the same workflow definition; only the surface syntax differs. Omitting a provider in `run()` uses the default in-process agent backend. The runtime executes steps in order and returns camelCase fields on the result object (`runId`, `stepCount`, `traceEvents`).

For browser production embeds, prefer `@arcflow/static` over bundling the full SDK. See [Static site chatbot](paths/static-site-chatbot.md).

## Install

### From npm (consumer projects)

```bash
npm install arcflow
```

The package ships a prebuilt native `.node` binary and has no production npm dependencies.

### From this repository (local development)

```bash
cd sdk-typescript
npm install
npm run build
```

Verify:

```bash
node --input-type=module -e "import { Agent, Workflow } from './index.js'; console.log('import ok')"
```

## Example: agent and workflow

Save as `quickstart.ts`:

```typescript
import { Agent, Workflow } from "arcflow";

const researcher = new Agent({
  name: "researcher",
  role: "research",
  instructions: "Research the given topic and list key facts.",
});

const writer = new Agent({
  name: "writer",
  role: "write",
  instructions: "Turn the research into a short paragraph.",
});

const workflow = new Workflow({ name: "research_pipeline" });
workflow.step(researcher);
workflow.step(writer);

const result = await workflow.run("Analyze renewable energy trends");

console.log(result.output);
console.log(result.runId);
console.log(result.stepCount);
console.log(result.status);
```

Run with Node 18+ (top-level await) or wrap in `async function main()`:

```bash
node --experimental-vm-modules quickstart.ts
```

Or use `tsx`:

```bash
npx tsx quickstart.ts
```

The fluent `step()` pattern returns `this`, so you can chain:

```typescript
const wf = new Workflow({ name: "research_pipeline" })
  .step(researcher)
  .step(writer);

const result = await wf.run("Analyze renewable energy trends");
```

## Reading the result

| Field | Meaning |
|-------|---------|
| `output` | Final step text |
| `runId` | Run UUID (camelCase in TypeScript) |
| `stepCount` | Steps executed |
| `status` | Terminal status |
| `traceEvents` | Raw metadata events from the runtime |

```typescript
for (const event of result.traceEvents ?? []) {
  const row = event as { event_kind?: string; sequence?: number };
  console.log(row.event_kind, row.sequence);
}
```

Cross-language runs produce equivalent trace shapes; integration tests in `sdk-python/tests/integration/test_cross_language_equivalence.py` compare normalized trace status and step counts.

## Structured trace via `workflow.trace()`

```typescript
const trace = workflow.trace();
console.log(trace.summary());
console.log(trace.status);
console.log(trace.steps.length);
console.log(trace.totalTokensConsumed);
```

`trace()` before `run()` throws `TraceNotFoundError`.

## Optional: real LLM with OpenAI

```typescript
import { Agent, OpenAI, Workflow } from "arcflow";

const wf = new Workflow({ name: "demo" });
wf.step(
  new Agent({
    name: "writer",
    role: "author",
    instructions: "Summarize in three sentences.",
  }),
);

const result = await wf.run("Quantum networking", {
  provider: new OpenAI({ model: "gpt-4o" }),
});

console.log(result.output);
```

Set `OPENAI_API_KEY` in the environment before running. Anthropic and Gemini providers follow the same `run()` options pattern with `ANTHROPIC_API_KEY` and `GEMINI_API_KEY`.

See [Provider configuration](../guides/agents-and-tools/provider-configuration.md).

## Async patterns

All execution entry points return Promises. Use `async`/`await` in scripts:

```typescript
async function main() {
  const result = await new Workflow({ name: "demo" })
    .step(new Agent({ name: "a", role: "researcher", instructions: "Work." }))
    .run("input");
  console.log(result.output);
}

main().catch(console.error);
```

For streaming in Node, see `runStream()` on `Workflow`. See [SDK streaming](../guides/streaming/sdk-streaming.md).

## Common errors

| Error | Typical cause |
|-------|----------------|
| `WorkflowConfigurationError` | Invalid workflow name, graph/step mismatch, bad agent type |
| `WorkflowExecutionError` | Runtime step failure |
| `TraceNotFoundError` | No completed run on this workflow instance |

## Equivalence with Python

| Python | TypeScript |
|--------|------------|
| `Workflow("name")` | `new Workflow({ name: "name" })` |
| `workflow.step(agent)` | `workflow.step(agent)` |
| `workflow.run("input")` | `await workflow.run("input")` |
| `result.run_id` | `result.runId` |
| `OpenAI(model="gpt-4o")` | `new OpenAI({ model: "gpt-4o" })` |

## Verify

| Check | Expected |
|-------|----------|
| Import succeeds | `import ok` |
| Default run (no API key) | `stepCount === 2`, non-empty `output` |
| `workflow.trace()` after run | Summary prints without exception |

## Next

| Topic | Link |
|-------|------|
| Python twin | [Python quickstart](quickstart-python.md) |
| Track A with verification steps | [Track A: First workflow](../tutorials/track-a-first-workflow.md) |
| Provider guide | [Provider configuration](../guides/agents-and-tools/provider-configuration.md) |
| Browser static client | [Static site chatbot](paths/static-site-chatbot.md) |
| VS Code extension | [VS Code overview](../vscode/overview.md) |
