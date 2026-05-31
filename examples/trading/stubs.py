# Demo tool stubs for paper-trading workflow.

from __future__ import annotations

import json
import os

PAPER = os.environ.get("PAPER_TRADING", "true").lower() == "true"


def place_order(args: dict[str, object]) -> str:
    symbol = str(args.get("symbol", ""))
    side = str(args.get("side", "buy"))
    qty = int(args.get("qty", 1))
    if not PAPER:
        return json.dumps({"error": "Live trading disabled — set PAPER_TRADING=true for demo"})
    return json.dumps({"paper": True, "symbol": symbol, "side": side, "qty": qty, "status": "accepted"})


def web_search_stub(args: dict[str, object]) -> str:
    query = str(args.get("query", ""))
    return json.dumps(
        {
            "stub": True,
            "query": query,
            "headlines": ["Services beat expectations", "50-DMA hold continues"],
        }
    )
