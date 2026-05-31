"""Clinical protocol Q&A with de-identified KB and disclaimer step (Phase 2-Pro)."""

from __future__ import annotations

from arcflow import Agent, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, Workflow
from arcflow.hitl import HitlConfig
from arcflow.memory import VectorStore

KB = """Protocol A: Standard hydration guidance for adults.
Protocol B: Escalation when symptoms persist beyond 48 hours.
Disclaimer: Not medical advice — consult a licensed clinician."""


def main() -> None:
    store = VectorStore()
    store.ingest("clinical-kb", "protocols", KB)
    qa = Agent(
        name="protocol_qa",
        role="clinical_assistant",
        instructions="Answer from the knowledge base only. Always include disclaimer.",
        memory=MemoryConfig(
            MemoryType.VECTOR,
            MemoryScope.AGENT,
            namespace="clinical-kb",
            embedding="stub/384",
            retrieval=MemoryRetrievalConfig(top_k=3),
        ),
    )
    reviewer = Agent(
        name="reviewer",
        role="reviewer",
        instructions="Confirm disclaimer is present before release.",
    )
    wf = (
        Workflow("protocol_qa")
        .step(qa)
        .step(reviewer, hitl=HitlConfig(approval_key="clinical_disclaimer", timeout_seconds=300))
    )
    print(wf.run("Summarize Protocol A.").output)


if __name__ == "__main__":
    main()
