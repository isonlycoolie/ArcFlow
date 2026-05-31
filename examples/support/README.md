# Tier-1 Support Ticket Assistant

## Problem

**Acme SaaS** receives 400+ tickets per week. Sixty percent are repeat issues: password resets, SSO certificate rotation, and seat provisioning. L1 agents copy-paste the same Confluence snippets while SLAs slip.

The support ops team wants a bot that retrieves **resolved ticket patterns** and draft replies, grounded in an internal KB, not hallucinated fixes.

## Who this is for

| Role | Goal |
|------|------|
| **Support lead** | Faster first response on known issues |
| **Platform engineer** | Hybrid RAG with persistent namespace per product line |
| **Security** | KB stays server-side; no customer PII in ingest text |

## What ArcFlow demonstrates

- **Persistent** vector namespace (`support-acme-prod`) survives across runs
- **Hybrid retrieval** + local rerank stub
- Single-agent workflow simulating ticket triage

## Prerequisites

- Python SDK installed
- For production: Qdrant + real embeddings

## Run

```bash
python examples/support/l1_ticket_copilot.py
```

Custom ticket description:

```bash
python examples/support/l1_ticket_copilot.py "Users get SAML error after IdP cert rotation"
```

## Verify

- Output references SSO / IdP certificate steps from [`data/ticket_kb.md`](data/ticket_kb.md)
- Suggested steps match Ticket #1002 pattern (certificate rotation)
- No fabricated admin URLs or credentials

## Production notes

- Ingest from Zendesk/Intercom exports; strip PII before ingest
- Wire to server `POST /v1/runs` for audited traces
- Escalation path: low confidence, then human queue (add HITL step)

## Files

| File | Purpose |
|------|---------|
| [`l1_ticket_copilot.py`](l1_ticket_copilot.py) | Support copilot workflow |
| [`data/ticket_kb.md`](data/ticket_kb.md) | Resolved ticket patterns |
