# ArcFlow TypeScript SDK

Promise-native workflow orchestration backed by the same Rust runtime as the Python SDK.

## Install

```bash
npm install arcflow
```

Local development:

```bash
npm install
npm run build
```

Requires Node.js 18+.

## Quick start

```typescript
import { Agent, OpenAI, Workflow } from "arcflow";

const wf = new Workflow({ name: "research" });
wf.step(
  new Agent({
    name: "writer",
    role: "author",
    instructions: "Write a concise summary.",
  }),
);

const result = await wf.run("Quantum networking", {
  provider: new OpenAI({ model: "gpt-4o" }),
});
console.log(result.output);

const trace = wf.trace();
console.log(trace.steps.length, trace.totalTokensConsumed);
```

Set `OPENAI_API_KEY` (or Anthropic/Gemini equivalents) in the environment.

## Documentation

- [Getting started](../contracts/guides/sdks/typescript/getting-started.md)
- [Provider guide](../contracts/guides/providers/getting-started.md)
- [Runtime contracts](../contracts/README.md)

## Zero production npm dependencies

The published package ships a prebuilt native `.node` binary; TypeScript sources compile to plain JavaScript with no runtime npm deps.
