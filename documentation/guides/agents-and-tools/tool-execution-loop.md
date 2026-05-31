
# Tool execution loop

When an agent defines tools, the runtime runs an iterative loop: send schemas to the provider, accept tool calls or final text, validate inputs, execute tools, feed results back, repeat until completion or `max_iterations`. Default mode is `llm_select`; legacy `legacy_eager` exists for compatibility.

Agent setup: [Defining agents](defining-agents.md). Execution context: [Execution model](../../concepts/execution-model.md).

## ToolExecutionConfig

```json
{
  "tool_execution": {
    "mode": "llm_select",
    "max_iterations": 5
  }
}
```

| Mode | Behavior |
|------|----------|
| `llm_select` | Model chooses whether to call tools each turn (default) |
| `legacy_eager` | Older eager execution path; prefer `llm_select` for new workflows |

Default max iterations is enforced by the engine. When exceeded, the step fails with an appropriate error code (often `StepExecutionFailed`).

## Loop sequence (LlmSelect)

```text
1. AgentInvoked
2. ProviderRequestSent (tools in request)
3. ProviderResponseReceived
   a. If text completion → AgentResponseReceived → step done
   b. If tool calls → for each call:
      - ToolCallStarted
      - Validate input against JSON Schema
      - On validation fail → ToolInputValidationFailed
      - Execute tool handler
      - ToolCallCompleted or ToolCallFailed
      - Feed result to model
4. Repeat from step 2 until text or max_iterations
5. TokensConsumed (may appear per iteration)
```

## Tool definition and validation

```json
{
  "name": "get_weather",
  "input_schema": {
    "type": "object",
    "properties": {
      "city": { "type": "string" },
      "units": { "type": "string", "enum": ["celsius", "fahrenheit"] }
    },
    "required": ["city"]
  }
}
```

Invalid call example the runtime rejects:

```json
{
  "city": 42
}
```

Trace:

```json
{
  "kind": "ToolInputValidationFailed",
  "run_id": "r1",
  "step_id": "s1",
  "tool_name": "get_weather",
  "violation_description": "city: expected string"
}
```

## Example trace for successful tool call

```json
[
  { "kind": "ToolCallStarted", "run_id": "r1", "step_id": "s1", "tool_name": "search_kb", "input_schema_hash": "abc123" },
  { "kind": "ToolCallCompleted", "run_id": "r1", "step_id": "s1", "tool_name": "search_kb", "duration_ms": 45, "output_size_bytes": 512 },
  { "kind": "ProviderRequestSent", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "model_id": "gpt-4o-mini", "prompt_size_bytes": 2048 },
  { "kind": "ProviderResponseReceived", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "tokens": { "input": 300, "output": 80, "total": 380 }, "latency_ms": 1100 }
]
```

SEC-1: traces record schema hashes and byte sizes, not tool arguments or results. See [SEC-1 and data safety](../../concepts/sec-1-and-data-safety.md).

## Python tool registration

Examples use `@tool` decorator patterns:

```python
from arcflow import Agent, tool, Workflow

@tool
def search_kb(query: str) -> str:
    """Search the knowledge base."""
    return f"Results for: {query}"

agent = Agent(
    name="researcher",
    role="Analyst",
    instructions="Use search_kb when you need facts.",
    tools=[search_kb],
    tool_execution={"mode": "llm_select", "max_iterations": 5},
)
```

## Multi-tool turns

The model may request multiple tools in one provider response. The runtime executes each validated call before the next provider round. Order is deterministic within the engine implementation; do not rely on cross-tool side effects for correctness.

## Provider errors during tool loop

| Event | Meaning |
|-------|---------|
| `ProviderRateLimited` | Backoff or retry per [Retry and backoff](../reliability/retry-and-backoff.md) |
| `ProviderError` | Terminal if not recovered |
| `ToolCallFailed` | Handler threw or returned error |

Rate limits may include `retry_after_seconds` in trace metadata.

## Testing tool loops

Use stub provider and test mode to avoid live API:

```json
{
  "exec_config": {
    "test": {
      "steps": {
        "s1": { "output": "Final answer without live tools" }
      }
    }
  }
}
```

For integration tests with real tool validation, use minimal schemas and stub handlers in SDK examples under `examples/`.

## Related pages

- [Defining agents](defining-agents.md)
- [Provider configuration](provider-configuration.md)
- [Validation and testing](../workflows/validation-and-testing.md)
- [Vector RAG pipeline](../memory-and-rag/vector-rag-pipeline.md) (retrieval as memory, not custom tools)
