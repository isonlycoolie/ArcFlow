# LangChain / LangGraph Migration

## Problem

Your team built prototypes with **LangChain** prompts, tools, and **LangGraph** state machines. Leadership wants a **self-hosted Rust runtime** with recovery, SEC-1 traces, and a single deployment artifact, without rewriting every prompt from scratch on day one.

ArcFlow provides adapters to import common LangChain shapes and heuristically map LangGraph compiled graphs to ArcFlow graph workflows.

## Who this is for

| Role | Goal |
|------|------|
| **Migration engineer** | Prove adapter path before full rewrite |
| **Tech lead** | Compare linear vs graph import side-by-side |

## What ArcFlow demonstrates

- `FromLangChain.prompt()` from LangChain `PromptTemplate`
- `FromLangChain.tool()` for duck-typed tools
- `LangChainToArcflow.convert()` heuristic on a mock compiled graph

## Prerequisites

```bash
# Install the SDK for development with LangChain extras
pip install -e "sdk-python[langchain]"
# or: pip install langchain-core langchain-community

# Or install the published SDK from PyPI for normal use (no extras):
pip install arcflow-sdk
```

## Run

```bash
python examples/langchain/langchain_adapter_demo.py
```

If extras missing, script prints install hint and exits cleanly.

## Verify

- Prints linear workflow step count
- Prints graph workflow node count and entry node
- No uncaught import errors when extras installed

## Production notes

- Heuristic graph import requires human review of edges and conditions
- Prefer native ArcFlow RCS JSON for production registry publishes
- See [graph/](../graph/) for idiomatic graph authoring

## Files

| File | Purpose |
|------|---------|
| [`langchain_adapter_demo.py`](langchain_adapter_demo.py) | Adapter showcase |
