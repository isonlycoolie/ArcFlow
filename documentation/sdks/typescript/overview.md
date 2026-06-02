# TypeScript SDK overview


The ArcFlow TypeScript SDK exposes the same Rust runtime as Python through an N-API native module. APIs are Promise-based and follow Node.js naming (camelCase fields on results, `async/await` for runs).

Use this SDK for Node.js backends, integration tests, the VS Code extension, and any server-side workflow orchestration. For production browser chat on static sites, prefer `@arcflow/static` and Relay rather than bundling this native module.

## What you can build

| Capability | TypeScript surface | Parity note |
|------------|-------------------|-------------|
| Linear workflows | `new Workflow({ name })` + `step()` | Full |
| Graph workflows | `Workflow({ graph: true })` + `node()`, `addEdge()`, `joinNode()` | Full |
| LLM providers | `OpenAI`, `Anthropic`, `Gemini` | Full |
| Recovery | `enableRecovery()` | Full for linear; graph resume partial (Graph recovery resume) |
| HITL | `HitlConfig` on `step()` | Full |
| Streaming | `runStream()` | In-process; not with remote runtime |
| Vector ingest/search | `VectorStore`, `ChunkHit` | Full |
| External bindings | `externalBinding()`, `ExternalOutcomeReport` type | Types only; no `reportOutcome()` helper in SDK |
| Testing helpers | `buildTestExecConfig`, `enableStubMode` | Vitest-oriented |
| Registry / remote | `runtime` in config, `publish()`, `resolve()` | Full |
| Tools on Agent | not exposed | Python only today |
| Memory on Agent | not exposed | Python only today |
| LangChain adapter | not present | Python `arcflow.langchain` only |
| Schedule manifest | not present | Python `ScheduleManifest` only |

## Architecture

```
Your Node.js / TS app
 |
 v
arcflow package (index.ts + index.native.js)
 |
 v
arcflow-core (Rust, same as Python)
```

The published npm package ships a prebuilt `.node` binary. Local development in this repo runs `npm run build` to compile TypeScript and the native binding.

## Typical workflow

```typescript
import { Agent, OpenAI, Workflow } from "arcflow";

const wf = new Workflow({ name: "research_pipeline" });
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
console.log(result.runId);

const trace = wf.trace();
console.log(trace.steps.length, trace.totalTokensConsumed);
```

Omitting `provider` in `run()` uses the stub agent, matching Python behavior.

## Naming differences from Python

| Python | TypeScript |
|--------|------------|
| `run_id` | `runId` |
| `step_count` | `stepCount` |
| `trace_events` | `traceEventsJson` on stream result; parsed via `trace()` |
| `enable_recovery()` | `enableRecovery()` |
| `run_stream()` | `runStream()` |
| `max_iterations()` | `withMaxIterations()` |
| `retry()` | `withRetry()` |

## Package exports

Public API is defined in `sdk-typescript/index.ts`. See [API reference](api-reference.md) for the full list.

## Parity and gaps

TypeScript matches Python for core execution paths documented in [parity matrix](../parity-matrix.md). Gaps that matter in practice:

| Gap | Workaround |
|-----|------------|
| No `Agent` tools/memory in TS binding | Define workflows in Python or post workflow JSON to server |
| No LangChain module | Use Python adapter or manual workflow specification conversion |
| No `reportOutcome()` client | POST to server external callback with fetch + HMAC |
| Graph recovery resume incomplete | Same Graph recovery resume limitation as Python |
| Server SSE deferred (streaming deferred) | Use `runStream()` in-process or poll GET run |

## Related pages

| Page | Content |
|------|---------|
| [Installation](installation.md) | npm and local build |
| [API reference](api-reference.md) | Exported symbols |
| [Exception reference](exception-reference.md) | Error classes and `mapNativeError` |
| [TypeScript quickstart](../../getting-started/quickstart-typescript.md) | First run |
| [Parity matrix](../parity-matrix.md) | Cross-surface comparison |
