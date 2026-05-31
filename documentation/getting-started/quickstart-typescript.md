# TypeScript quickstart

**Audience:** `[developer]`

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

