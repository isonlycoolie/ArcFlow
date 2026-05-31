# ArcFlow Memory & Retrieval Guide

Source: internal platform wiki (excerpt)

## Vector memory overview

ArcFlow vector memory stores embedded document chunks in Qdrant (or in-memory for dev). Each agent declares a `namespace` so collections do not leak across teams.

## Chunking

Long documents should be split before ingest:

- **Default chunk size:** 512 tokens for general prose; **256** for API reference with dense tables
- **Overlap:** 32–64 tokens to preserve sentence boundaries across chunks
- Poor chunking causes "right doc, wrong paragraph" retrieval failures

## Hybrid retrieval

Hybrid mode combines dense (embedding) and sparse (BM25-style) signals:

- `dense_weight` + `sparse_weight` should sum to ~1.0
- Start with 0.65 / 0.35 for mixed FAQ + narrative docs
- Enable `ARCFLOW_QDRANT_HYBRID=true` on the server

## Rerank

Optional second stage reduces `top_k` candidates to the best `top_n` for prompt injection:

- Cohere rerank when `COHERE_API_KEY` is set
- Local rerank stub available for tests

## Operational checklist

1. Version your namespace on major doc restructures
2. Strip secrets and PII before ingest
3. Monitor `MemoryRetrieved` trace bytes — sudden drops may indicate index drift
