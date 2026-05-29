# TypeScript SDK — Getting Started

Install (local development):

```bash
cd sdk-typescript
npm install
npm run build
```

Quick start:

```typescript
import { Agent, Workflow } from "arcflow";

const wf = new Workflow({ name: "demo" });
wf.step(
  new Agent({
    name: "writer",
    role: "author",
    instructions: "Reply in one sentence.",
  }),
);

const result = await wf.run("Hello ArcFlow");
console.log(result.output, result.stepCount);

const trace = wf.trace();
console.log(trace.summary?.() ?? trace.status);
```

## Provider on `run()`

```typescript
import { OpenAI } from "arcflow";

await wf.run("topic", {
  provider: new OpenAI({ model: "gpt-4o" }),
});
```

Set `OPENAI_API_KEY` in the environment. Credentials are never passed in code.

## Errors

All errors extend `ArcFlowError` and use the `[ArcFlow]` message prefix. Catch with `instanceof`:

```typescript
import { WorkflowConfigurationError } from "arcflow";
```

See the [TypeScript SDK README](../../sdk-typescript/README.md) for the full public API.
