"""Research + RAG + paper trade pipeline (Phase 2-Pro)."""

from __future__ import annotations

import json
import os

from arcflow import Agent, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, Tool, Workflow
from arcflow.hitl import HitlConfig
from arcflow.memory import VectorStore

PAPER = os.environ.get("PAPER_TRADING", "true").lower() == "true"
KB = "AAPL swing thesis: momentum above 50DMA with tight stop."


def place_order(args: dict[str, object]) -> str:
    symbol = str(args.get("symbol", ""))
    side = str(args.get("side", "buy"))
    qty = int(args.get("qty", 1))
    if not PAPER:
        return json.dumps({"error": "Live trading disabled — set PAPER_TRADING=true"})
    return json.dumps({"paper": True, "symbol": symbol, "side": side, "qty": qty})


def web_search_stub(args: dict[str, object]) -> str:
    query = str(args.get("query", ""))
    return json.dumps({"stub": True, "query": query})


def main() -> None:
    store = VectorStore()
    store.ingest("trade-kb", "notes", KB)
    researcher = Agent(
        name="researcher",
        role="researcher",
        instructions="Gather market context.",
        tools=(
            Tool(
                "web_search",
                "Search for market news.",
                {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]},
                web_search_stub,
            ),
        ),
    )
    analyst = Agent(
        name="analyst",
        role="analyst",
        instructions="Analyze using RAG notes.",
        memory=MemoryConfig(
            MemoryType.VECTOR,
            MemoryScope.WORKFLOW,
            namespace="trade-kb",
            embedding="stub/384",
            retrieval=MemoryRetrievalConfig(top_k=3),
        ),
    )
    strategist = Agent(name="strategist", role="strategist", instructions="Propose trade plan.")
    executor = Agent(
        name="executor",
        role="executor",
        instructions="Place paper order when approved.",
        tools=(
            Tool(
                "place_order",
                "Place a paper trade order.",
                {
                    "type": "object",
                    "properties": {
                        "symbol": {"type": "string"},
                        "side": {"type": "string"},
                        "qty": {"type": "integer"},
                    },
                    "required": ["symbol"],
                },
                place_order,
            ),
        ),
    )
    wf = (
        Workflow("auto_research_trade")
        .step(researcher)
        .step(analyst)
        .step(strategist)
        .step(executor, hitl=HitlConfig(approval_key="trade_execute", timeout_seconds=120))
    )
    print(wf.run("Analyze AAPL for a small swing trade").output)


if __name__ == "__main__":
    main()
