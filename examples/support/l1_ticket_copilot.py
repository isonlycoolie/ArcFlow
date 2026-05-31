# L1 support copilot — hybrid RAG over resolved ticket playbooks.

from __future__ import annotations

import sys
from pathlib import Path

from arcflow import Agent, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, VectorStore, Workflow

NAMESPACE = "support-acme-prod"
KB_PATH = Path(__file__).parent / "data" / "ticket_kb.md"
DEFAULT_TICKET = "Customer cannot log in after SSO certificate rotation on their IdP."


def main() -> None:
    ticket = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_TICKET
    kb = KB_PATH.read_text(encoding="utf-8")

    store = VectorStore()
    store.ingest(NAMESPACE, "resolved_patterns", kb)

    bot = Agent(
        name="l1_copilot",
        role="support_engineer",
        instructions=(
            "Draft a concise L1 reply using only the knowledge base. "
            "Include numbered troubleshooting steps when available. "
            "If the issue matches billing, say escalate to L2 with account ID. "
            "Never invent URLs or promise refunds."
        ),
        memory=MemoryConfig(
            MemoryType.VECTOR,
            MemoryScope.PERSISTENT,
            namespace=NAMESPACE,
            embedding="stub/384",
            retrieval=MemoryRetrievalConfig(mode="hybrid", rerank="local", top_k=4),
        ),
    )
    wf = Workflow("acme_l1_ticket_copilot").step(bot)
    print(wf.run(ticket).output)


if __name__ == "__main__":
    main()
