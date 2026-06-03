# Research to Paper Trade Pipeline

## Problem

A small prop desk runs **discretionary swing trades** on a short watchlist. Analysts manually:

1. Scan news and filings
2. Cross-check internal research notes
3. Write a one-page thesis
4. Wait for a senior trader to approve before any order hits the (paper) book

They want automation with a **hard human gate** before execution, and **no live trading** in the demo environment.

## Who this is for

| Role | Goal |
|------|------|
| **Analyst** | Faster research synthesis |
| **Senior trader** | Approve/reject before orders |
| **Engineering** | Multi-step workflow + tools + HITL + RAG |

## What ArcFlow demonstrates

- Linear pipeline: researcher, then analyst (RAG), strategist, executor
- **Tools:** stub web search, paper `place_order`
- **HITL** on executor step (`trade_execute`)
- `PAPER_TRADING=true` env guard (default)

## Prerequisites

```bash
# Install the SDK for development (editable)
pip install -e sdk-python

# Or install the published SDK from PyPI for normal use:
pip install arcflow-sdk

export PAPER_TRADING=true   # default; never set false in this example
```

For HITL interrupt/resume against server:

```bash
docker compose -f docker/docker-compose.server.yml up -d
```

## Run

```bash
python examples/trading/research_to_paper_trade.py
```

## Verify

- Workflow completes four steps (or interrupts at executor with server recovery)
- Order tool returns `"paper": true` JSON only
- Analyst step retrieves thesis notes from [`data/research_notes.md`](data/research_notes.md)

## Production notes

- **Not financial advice**, demo only; real trading needs compliance review
- Live brokers require separate integration; keep HITL mandatory
- Store research in versioned namespaces per desk

## Files

| File | Purpose |
|------|---------|
| [`research_to_paper_trade.py`](research_to_paper_trade.py) | Full pipeline |
| [`data/research_notes.md`](data/research_notes.md) | Internal watchlist notes |
