**Audience:** `[developer]`

# Defining agents

Agents are RCS `AgentDefinition` objects. Workflow steps reference them by UUID via `agent_id`. Each agent carries instructions, optional tools, memory, context policy, tool execution settings, and provider configuration. Surfaces (Python, TypeScript, server) serialize the same JSON shape to `arcflow-core`.

Start with [First workflow in five minutes](../../getting-started/first-workflow-in-five-minutes.md) for a minimal two-agent linear run. Type definitions: [The RCS contract](../../concepts/the-rcs-contract.md).

## AgentDefinition fields

| Field | Purpose |
|-------|---------|
| `id` | UUID; must match step `agent_id` |
| `name`, `role` | Trace labels and prompt framing |
| `instructions` | System or task prompt |
| `tools` | Optional `ToolDefinition[]` with JSON Schema inputs |
| `memory_config` | Session, shared, persistent, or vector backend |
| `context` | `ContextPolicy` for prior steps and run input |
| `tool_execution` | `llm_select` (default) or `legacy_eager`; max iterations |
| `provider` | `ProviderConfig`: provider_id, model, api_key_env |

## Full example with tool and vector memory

```json
{
  "id": "00000000-0000-4000-8000-000000000020",
  "name": "researcher",
  "role": "Research analyst",
  "instructions": "Use search_kb for facts. Cite sources.",
  "tools": [
    {
      "name": "search_kb",
      "input_schema": {
        "type": "object",
        "properties": {
          "query": { "type": "string" }
        },
        "required": ["query"]
      }
    }
  ],
  "memory_config": {
    "memory_type": "vector",
    "scope": "workflow",
    "namespace": "acme-support",
    "embedding": "openai/text-embedding-3-small",
    "retrieval": {
      "mode": "hybrid",
      "top_k": 5,
      "dense_weight": 0.7,
      "sparse_weight": 0.3,
      "rerank": {
        "provider": "cohere",
        "model": "rerank-english-v3.0",
        "top_n": 3
      }
    }
  },
  "context": {
    "include_prior_steps": "last",
    "include_run_input": true,
    "max_prior_step_chars": 4096
  },
  "tool_execution": {
    "mode": "llm_select",
    "max_iterations": 5
  },
  "provider": {
    "provider_id": "openai",
    "model": "gpt-4o-mini",
    "api_key_env": "OPENAI_API_KEY"
  }
}
```

Memory details: [Memory types](../memory-and-rag/memory-types.md). Context: [Context policies](context-policies.md). Tools: [Tool execution loop](tool-execution-loop.md).

## ToolDefinition

```json
{
  "name": "search_kb",
  "input_schema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Search query" },
      "top_k": { "type": "integer", "minimum": 1, "maximum": 20 }
    },
    "required": ["query"]
  },
  "permissions": []
}
```

Runtime validates tool call arguments against `input_schema`. Violations emit `ToolInputValidationFailed` and map to `ToolExecutionFailed` on terminal failure.
