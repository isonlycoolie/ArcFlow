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

| Event | When |
|-------|------|
| `ProviderRequestSent` | Outbound LLM request (metadata only) |
| `ProviderResponseReceived` | Success with token counts and latency |
| `ProviderRateLimited` | 429 or provider rate signal |
| `ProviderError` | Terminal provider failure |

Example:

```json
{
  "kind": "ProviderResponseReceived",
  "run_id": "r1",
  "step_id": "s1",
  "provider_id": "openai",
  "model_id": "gpt-4o-mini",
  "tokens": { "input": 120, "output": 45, "total": 165 },
  "latency_ms": 890
}
```

## Error mapping

| ErrorCode | HTTP (server) | Typical cause |
|-----------|---------------|---------------|
| `ProviderError` | 502 | API error, invalid model |
| `RateLimited` | 429 | Provider or site rate limit |
| `EmbeddingError` | 502 | Embedding call failed |
| `RerankError` | 502 | Cohere rerank failed |

Appendix E lists all RCS error codes.

## Server deployment

Set keys in the server environment or secrets manager, not in Postgres:

```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export COHERE_API_KEY=...   # rerank only
```

Docker Compose: `docker/docker-compose.server.yml` documents the expected service env block.

## Security rules

- No API keys in RCS JSON, registry payloads, or static JS bundles
- Relay and static SDK never see LLM keys; server holds them
- Logs and traces are SEC-1 metadata only ([SEC-1 and data safety](../../concepts/sec-1-and-data-safety.md))

## Python and TypeScript shorthand

Python SDK often accepts `provider="openai/gpt-4o-mini"` with separate `api_key_env`.

TypeScript:

```typescript
provider: {
  providerId: "openai",
  model: "gpt-4o-mini",
  apiKeyEnv: "OPENAI_API_KEY",
}
```

## Related pages

- [Tool execution loop](tool-execution-loop.md)
- [Hybrid retrieval and reranking](../memory-and-rag/hybrid-retrieval-and-reranking.md) (Cohere rerank)
- [Retry and backoff](../reliability/retry-and-backoff.md) (ProviderRateLimited)
- [Python quickstart](../../getting-started/quickstart-python.md)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §5.3; Appendix A (ProviderConfig); Appendix E (ProviderError, EmbeddingError, RerankError); Appendix H (env vars).
