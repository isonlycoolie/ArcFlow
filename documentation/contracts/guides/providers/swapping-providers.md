# Swapping Providers

The same workflow definition works with any supported provider, pass the provider only at `run()` time (ADR-010).

## Python

```python
from arcflow import Agent, Anthropic, Gemini, OpenAI, Workflow

agent = Agent(name="writer", role="author", instructions="One paragraph.")
wf = Workflow("swap-demo").step(agent)

wf.run("topic", provider=OpenAI(model="gpt-4o"))
wf.run("topic", provider=Anthropic(model="claude-3-5-sonnet-20241022"))
wf.run("topic", provider=Gemini(model="gemini-1.5-pro"))
```

## TypeScript

```typescript
import { Anthropic, Gemini, OpenAI } from "arcflow";

await wf.run("topic", { provider: new OpenAI({ model: "gpt-4o" }) });
await wf.run("topic", { provider: new Anthropic({ model: "claude-3-5-sonnet-20241022" }) });
await wf.run("topic", { provider: new Gemini({ model: "gemini-1.5-pro" }) });
```

## Trace equivalence

Successful runs produce the same `step_count` and structurally equivalent traces (provider-specific token counts may differ). CI validates this with mock HTTP servers, no live API keys required.

## Stub fallback

`workflow.run(input)` without a provider continues to use the stub agent for backwards compatibility.
