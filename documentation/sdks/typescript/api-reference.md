# TypeScript SDK API reference

**Audience:** `[developer]`

Exports are defined in `sdk-typescript/index.ts`. This page lists every public symbol and notes parity gaps against the Python SDK.

```typescript
import { Agent, Workflow, OpenAI } from "arcflow";
```

## Workflow

`new Workflow(config?: WorkflowConfig)`

```typescript
interface WorkflowConfig {
  name?: string;      // default "default"
  graph?: boolean;    // default false
  runtime?: string;   // server base URL for remote runs
}
```

| Method | Description |
|--------|-------------|
| `step(agent, options?: { hitl?: HitlConfig })` | Append linear step |
| `node(nodeId, agent)` | Register graph node (`graph: true`) |
| `addEdge(fromId, toId?, options?: { condition? })` | Graph edge |
| `joinNode(joinId, waitFor)` | Join waiting for branches |
| `setEntry(nodeId)` | Set graph entry |
| `withMaxIterations(count)` | Graph iteration cap |
| `withRetry(maxAttempts, options?)` | Retry before run |
| `withTimeout(seconds)` | Workflow timeout |
| `withStepTimeout(seconds)` | Per-step timeout |
| `enableRecovery()` | Postgres recovery flag |
| `run(input, options?: RunOptions)` | `Promise<WorkflowResult>` |
| `runStream(input, options?)` | Async iterable of `StreamEvent` |
| `resume(runId, options?)` | Resume failed run |
| `resumeWithApproval(runId, approvalKey, options?)` | HITL approve/reject |
| `trace()` | `TraceResult` for last run |
| `test(cases)` | Deterministic stub cases |
| `publish(version, options?)` | Publish to server |
| `static resolve(name, version, options)` | Load registry ref |

`RunOptions`:

```typescript
interface RunOptions {
  provider?: Provider;
  initialState?: Record<string, unknown>;
}
```

`runStream()` is not supported when `runtime` points at a remote server (same constraint as Python).

## Agent

`new Agent(config: AgentConfig)`

```typescript
interface AgentConfig {
  name: string;
  role: string;
  instructions: string;
  model?: string;  // default "default"
}
```

Readonly fields: `name`, `role`, `instructions`, `model`, `agentId`.

**Parity gap:** Python `Agent` accepts `tools`, `memory`, `context`, and `tool_execution`. The TypeScript `Agent` binding does not expose these yet. Tool and memory workflows in TypeScript today require server-side definitions or Python-authored RCS.

## Providers

All implement `bindingRow()` for the native layer.

| Class | Env var | Constructor |
|-------|---------|-------------|
| `OpenAI` | `OPENAI_API_KEY` | `new OpenAI({ model, maxTokens?, temperature? })` |
| `Anthropic` | `ANTHROPIC_API_KEY` | `new Anthropic({ model, maxTokens?, temperature? })` |
| `Gemini` | `GEMINI_API_KEY` | `new Gemini({ model, maxTokens?, temperature? })` |

Type alias: `Provider = OpenAI | Anthropic | Gemini`.

## WorkflowResult

| Field | Type |
|-------|------|
| `output` | `string` |
| `runId` | `string` |
| `stepCount` | `number` |
| `status` | `string` |
| `approvalKey` | `string \| undefined` |

Produced by `toWorkflowResult()` from native execution output.

