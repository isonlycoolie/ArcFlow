**Audience:** `[developer]` `[platform]`

# Provider configuration

LLM and embedding calls route through `ProviderConfig` on each agent. The runtime reads API keys from environment variables named in `api_key_env`, never from workflow JSON or browser bundles. Provider failures map to typed errors and trace events shared across SDK and server.

Agent field reference: [Defining agents](defining-agents.md). Install and env setup: [Install and build](../../getting-started/install-and-build.md).

## ProviderConfig shape

```json
{
  "provider": {
    "provider_id": "openai",
    "model": "gpt-4o-mini",
    "api_key_env": "OPENAI_API_KEY",
    "params": {
      "temperature": 0.3,
      "max_tokens": 4096
    }
  }
}
```

| Field | Role |
|-------|------|
| `provider_id` | Backend selector (see table below) |
| `model` | Provider-specific model id |
| `api_key_env` | Name of env var holding the secret |
| `params` | Optional provider-specific generation params |

## Supported chat providers

| provider_id | Env variable | Notes |
|-------------|--------------|-------|
| `openai` | `OPENAI_API_KEY` | Chat and embeddings |
| `anthropic` | `ANTHROPIC_API_KEY` | Claude models |
| `gemini` | `GEMINI_API_KEY` | Google models |
| `stub` | (none) | Tests only; deterministic responses |

### OpenAI example

```json
{
  "provider_id": "openai",
  "model": "gpt-4o-mini",
  "api_key_env": "OPENAI_API_KEY"
}
```

### Anthropic example

```json
{
  "provider_id": "anthropic",
  "model": "claude-3-5-sonnet-20241022",
  "api_key_env": "ANTHROPIC_API_KEY"
}
```

### Stub (local and CI)

```json
{
  "provider_id": "stub",
  "model": "stub-v1",
  "api_key_env": ""
}
```

[First workflow in five minutes](../../getting-started/first-workflow-in-five-minutes.md) uses stub implicitly when no provider is configured.

## Embedding provider strings

Vector memory uses a separate embedding string on `memory_config.embedding`, not `ProviderConfig`:

```json
{
  "memory_config": {
    "memory_type": "vector",
    "embedding": "openai/text-embedding-3-small",
    "namespace": "product-docs"
  }
}
```

Production also requires:

- `ARCFLOW_QDRANT_URL`
- `ARCFLOW_EMBEDDING_PROVIDER` (non-stub in prod)
- `OPENAI_API_KEY` (when embedding string uses OpenAI)

See [Vector RAG pipeline](../memory-and-rag/vector-rag-pipeline.md).

## Trace events
