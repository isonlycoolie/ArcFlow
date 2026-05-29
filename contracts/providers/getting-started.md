# Providers — Getting Started

ArcFlow supports OpenAI, Anthropic, and Gemini via `workflow.run(input, provider=...)`.

## Environment variables

| Provider | API key env var |
|----------|-----------------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Gemini | `GEMINI_API_KEY` |

Optional mock endpoint overrides (CI only):

- `ARCFLOW_OPENAI_API_ENDPOINT`
- `ARCFLOW_ANTHROPIC_API_ENDPOINT`
- `ARCFLOW_GEMINI_API_ENDPOINT`

## Python

```python
from arcflow import Agent, OpenAI, Workflow

wf = Workflow("demo")
wf.step(Agent(name="a", role="writer", instructions="Summarize."))
result = wf.run("topic", provider=OpenAI(model="gpt-4o"))
```

## TypeScript

```typescript
import { Agent, OpenAI, Workflow } from "arcflow";

const wf = new Workflow({ name: "demo" });
wf.step(new Agent({ name: "a", role: "writer", instructions: "Summarize." }));
await wf.run("topic", { provider: new OpenAI({ model: "gpt-4o" }) });
```

Without a provider, workflows use the deterministic stub agent (Sprint 2 behaviour).

See [PROVIDER-API-CONTRACT-v1.md](../PROVIDER-API-CONTRACT-v1.md) for HTTP mappings and error codes.
