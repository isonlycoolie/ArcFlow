# PROVIDER-API-CONTRACT v1

**Status:** Implemented (Sprint 6)  
**Scope:** `ModelProvider` trait and OpenAI / Anthropic / Gemini adapters in `arcflow-core`.

## ModelProvider trait

Object-safe, `Send + Sync + 'static`. All workflow execution uses `Arc<dyn ModelProvider>`.

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync + 'static {
    fn provider_id(&self) -> &str;  // "openai" | "anthropic" | "gemini"
    fn model_id(&self) -> &str;
    async fn complete(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderCallError>;
    async fn stream(&self, request: ProviderRequest) -> Result<ProviderStream, ProviderCallError>;
}
```

## ProviderRequest (internal — never traced)

| Field | Type | Notes |
|-------|------|-------|
| `messages` | `Vec<ProviderMessage>` | role + content — SEC-2 |
| `system_prompt` | `Option<String>` | agent instructions — SEC-2 |
| `max_tokens` | `u32` | |
| `temperature` | `f32` | 0.0–1.0 |

`prompt_size_bytes()` = sum of message + system byte lengths (safe for traces).

## ProviderResponse (internal — never traced)

| Field | Type | Trace-safe |
|-------|------|------------|
| `content` | `String` | No — step output only |
| `tokens` | `TokenUsage` | Yes |
| `model_id` | `String` | Yes |
| `finish_reason` | `FinishReason` | Yes |

## ProviderCallError → trace / RuntimeError

| Variant | HTTP / cause | Trace event |
|---------|--------------|-------------|
| `AuthenticationFailed` | 401 | `ProviderError` |
| `RateLimited` | 429 | `ProviderRateLimited` |
| `Timeout` | client timeout | `ProviderError` |
| `ApiError` | other 4xx/5xx | `ProviderError` (sanitized message) |
| `NotConfigured` | missing env key | `ProviderError` |
| `ContentFiltered` | safety block | `ProviderError` |
| `NetworkError` | connection | `ProviderError` |
| `ResponseParseError` | JSON | `ProviderError` |

## Per-provider mapping

### OpenAI

| Item | Value |
|------|-------|
| Endpoint | `OPENAI_API_ENDPOINT` constant |
| Auth | `Authorization: Bearer $OPENAI_API_KEY` |
| Key env | `OPENAI_API_KEY` |
| Rate limit | HTTP 429 |
| Tokens | `usage.prompt_tokens`, `usage.completion_tokens`, `usage.total_tokens` |
| Streaming | SSE (`eventsource-stream`) |

### Anthropic

| Item | Value |
|------|-------|
| Endpoint | `ANTHROPIC_API_ENDPOINT` |
| Auth | `x-api-key`, `anthropic-version` header |
| Key env | `ANTHROPIC_API_KEY` |
| Rate limit | HTTP 429 |
| Tokens | `usage.input_tokens`, `usage.output_tokens` |

### Gemini

| Item | Value |
|------|-------|
| Endpoint | `GEMINI_API_ENDPOINT` + `{model}:generateContent` |
| Auth | `x-goog-api-key` |
| Key env | `GEMINI_API_KEY` |
| Rate limit | HTTP 429 |
| Tokens | `usageMetadata` fields when present |

## Security (SEC-1/2/3)

- Credentials only from env at provider construction; never logged, traced, or in errors.
- Prompt/response content never in trace events — only sizes and token counts.
- Provider error bodies stripped to status + generic description when echo risk exists.

## RCS integration

SDKs pass [`ProviderConfig`](../rcs/v1.schema.json) on `RunRequest.provider_config`. `ProviderRuntime::from_config` builds `Arc<dyn ModelProvider>`.
