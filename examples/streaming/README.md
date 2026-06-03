# Streaming Chat Responses

## Problem

**Support chat widgets** feel broken when users stare at a spinner for 8 to 12 seconds. Product wants **token-by-token** rendering like ChatGPT, without exposing LLM keys in the browser.

The SDK streaming path lets a backend or embedded runner emit partial tokens while SEC-1 keeps traces metadata-only.

## Who this is for

| Role | Goal |
|------|------|
| **Frontend engineer** | Consume async token events |
| **Backend engineer** | Prototype `run_stream()` before server SSE (FP-2) |

## What ArcFlow demonstrates

- Python async `wf.run_stream()` event loop
- Event types: `token`, `step_start`, `step_complete`
- TypeScript variant: [`stream_support_reply.ts`](stream_support_reply.ts)

## Prerequisites

```bash
# Install the SDK for development (editable)
pip install -e sdk-python

# Or install the published SDK from PyPI for normal use:
pip install arcflow-sdk
```

## Run (Python)

```bash
python examples/streaming/stream_support_reply.py
```

## Run (TypeScript)

```bash
cd sdk-typescript && npm run build
npx ts-node ../examples/streaming/stream_support_reply.ts
```

## Verify

- Terminal prints incremental token text (stub path) or step events
- Event count > 0; script raises if stream is empty
- Trace contains `StreamChunkReceived` / `TokenEmitted` (metadata only)

## Production notes

- Browser production: static SDK polls Relay trace until server SSE ships
- Do not log token text in centralized observability (SEC-1)

## Files

| File | Purpose |
|------|---------|
| [`stream_support_reply.py`](stream_support_reply.py) | Python async streaming demo |
| [`stream_support_reply.ts`](stream_support_reply.ts) | TypeScript streaming demo |
