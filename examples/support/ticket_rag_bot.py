"""Support ticket bot with persistent namespace and rerank (Phase 2-Pro)."""

from __future__ import annotations

from arcflow import Agent, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, Workflow
from arcflow.memory import VectorStore

KB = """Ticket #1001: Reset password via Settings > Security.
Ticket #1002: SSO failures — verify IdP certificate rotation."""


def main() -> None:
    store = VectorStore()
    store.ingest("support-tickets", "kb", KB)
    bot = Agent(
        name="support_bot",
        role="support",
        instructions="Resolve tickets using the knowledge base.",
        memory=MemoryConfig(
            MemoryType.VECTOR,
            MemoryScope.PERSISTENT,
            namespace="support-tickets",
            embedding="stub/384",
            retrieval=MemoryRetrievalConfig(mode="hybrid", rerank="local", top_k=2),
        ),
    )
    wf = Workflow("ticket_rag_bot").step(bot)
    print(wf.run("Customer cannot log in after SSO change.").output)


if __name__ == "__main__":
    main()
