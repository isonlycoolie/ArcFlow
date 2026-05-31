# Product vision

## Problem

Static-site operators need to publish AI chat without embedding agents, API keys, or knowledge ingestion in frontend code. Today they can use OSS shell scripts; the dashboard replaces that with a guided UI backed by the same admin API.

## Personas

| Persona | Goal | Technical level |
|---------|------|-----------------|
| Landing-page builder | Add support chat to a marketing site in under 30 minutes | Low: knows env vars, not Rust |
| Platform operator | Manage multiple sites, origins, and published workflows | Medium: reads API docs |

## Product tiers

| Tier | Sites | Rate limit | Inline workflows | BYO LLM | Usage UI |
|------|-------|------------|------------------|---------|----------|
| Tier 1 (R1) | 1+ | 60 rpm default | Off (`allow_inline=false`) | Off | Off |
| Tier 2 (future) | Unlimited | Configurable | Optional | Vault-backed keys | Daily run chart |

## What the dashboard does

1. Creates Relay sites and shows one-time credentials (`VITE_ARCFLOW_RELAY_URL`, `VITE_ARCFLOW_SITE_TOKEN`)
2. Ingests knowledge text into the site vector namespace
3. Sets chat instructions and publishes the default `chat` workflow version
4. Edits allowed origins, rate limits, and rotates site tokens

## Non-goals (v1)

- ArcFlow Cloud hosting or billing
- Visual workflow graph editor
- End-user OAuth login to the dashboard (admin API key only in R1)
- Stripe or subscription UI
- Managed multi-region relay picker

## Success statement

An operator completes site setup, knowledge upload, and chat publish without running shell scripts or reading Rust source code.
