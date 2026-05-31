# Clinical Protocol Lookup (with HITL)

## Problem

A regional clinic network gives nurses a **de-identified protocol knowledge base** for common triage pathways. Nurses need fast answers ("What is Protocol A for adult hydration?") but leadership requires that **every patient-facing summary includes a disclaimer** and is reviewed before it leaves the assistant.

This is not a diagnostic system; it is internal staff lookup with a human safety gate.

## Who this is for

| Role | Goal |
|------|------|
| **Clinical ops** | Grounded answers from approved protocols only |
| **Compliance** | Mandatory disclaimer + human review step |
| **Engineering** | RAG + HITL on a single linear workflow |

## What ArcFlow demonstrates

- Vector ingest of protocol text
- Two-step workflow: QA agent, then reviewer with **HITL** (`approval_key=clinical_disclaimer`)
- Instructions that forbid answers outside the knowledge base

## Prerequisites

- Python SDK installed
- For live HITL interrupt/resume: `arcflow-server` + Postgres + `recovery_enabled`

```bash
docker compose -f docker/docker-compose.server.yml up -d
```

## Run

```bash
python examples/healthcare/staff_protocol_lookup.py
```

With server recovery, the run may **interrupt** at the reviewer step. Approve via API or [hitl/approve_cli.sh](../hitl/approve_cli.sh).

## Verify

- First-step output cites Protocol A/B content from [`data/protocols.md`](data/protocols.md)
- Disclaimer text appears or reviewer step triggers interrupt
- Trace shows `StepStarted` for both agents when HITL is disabled locally

## Production notes

- **Not a medical device**, for internal staff lookup only; always consult licensed clinicians
- Use production embedding + Qdrant; audit ingest sources
- Extend HITL timeout for clinical review queues

## Files

| File | Purpose |
|------|---------|
| [`staff_protocol_lookup.py`](staff_protocol_lookup.py) | Workflow with RAG + HITL gate |
| [`data/protocols.md`](data/protocols.md) | De-identified protocol excerpts |
