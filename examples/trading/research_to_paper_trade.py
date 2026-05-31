# Desk research-to-paper-trade pipeline with senior approval (PAPER_TRADING=true only).

from __future__ import annotations

import sys
from pathlib import Path

from arcflow import (
    Agent,
    HitlConfig,
    MemoryConfig,
    MemoryRetrievalConfig,
    MemoryScope,
    MemoryType,
    Tool,
    VectorStore,
    Workflow,
)

from stubs import place_order, web_search_stub

NAMESPACE = "desk-alpha-research"
NOTES_PATH = Path(__file__).parent / "data" / "research_notes.md"
DEFAULT_BRIEF = "Analyze AAPL for a small swing trade per desk notes."


def main() -> None:
    brief = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_BRIEF
    notes = NOTES_PATH.read_text(encoding="utf-8")

    store = VectorStore()
    store.ingest(NAMESPACE, "watchlist", notes)

    researcher = Agent(
        name="researcher",
        role="research_analyst",
        instructions="Gather market context for the requested symbol. Use web_search for headlines.",
        tools=(
            Tool(
                "web_search",
                "Search for recent market news.",
                {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]},
                web_search_stub,
            ),
        ),
    )
    analyst = Agent(
        name="analyst",
        role="equity_analyst",
        instructions="Analyze using internal research notes. Cite risk limits.",
        memory=MemoryConfig(
            MemoryType.VECTOR,
            MemoryScope.WORKFLOW,
            namespace=NAMESPACE,
            embedding="stub/384",
            retrieval=MemoryRetrievalConfig(top_k=3),
        ),
    )
    strategist = Agent(
        name="strategist",
        role="portfolio_strategist",
        instructions="Propose a paper trade plan: symbol, side, qty, stop rationale.",
    )
    executor = Agent(
        name="executor",
        role="execution",
        instructions="Place paper order only when plan is clear. Use place_order tool.",
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
        Workflow("research_to_paper_trade")
        .step(researcher)
        .step(analyst)
        .step(strategist)
        .step(executor, hitl=HitlConfig(approval_key="trade_execute", timeout_seconds=120))
    )
    print(wf.run(brief).output)


if __name__ == "__main__":
    main()
