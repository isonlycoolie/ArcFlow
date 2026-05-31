# Clinical staff protocol lookup — RAG plus HITL disclaimer review (not diagnostic).

from __future__ import annotations

from pathlib import Path

from arcflow import Agent, HitlConfig, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, VectorStore, Workflow

KB_NAMESPACE = "regional-clinic-protocols-v3"
PROTOCOLS_PATH = Path(__file__).parent / "data" / "protocols.md"


def main() -> None:
    kb_text = PROTOCOLS_PATH.read_text(encoding="utf-8")
    store = VectorStore()
    store.ingest(KB_NAMESPACE, "protocols", kb_text)

    lookup = Agent(
        name="staff_protocol_lookup",
        role="clinical_assistant",
        instructions=(
            "Answer from the knowledge base only. Summarize the relevant protocol steps. "
            "Always include: 'Not medical advice — consult a licensed clinician.' "
            "If the question is outside the KB, refuse and suggest physician consult."
        ),
        memory=MemoryConfig(
            MemoryType.VECTOR,
            MemoryScope.AGENT,
            namespace=KB_NAMESPACE,
            embedding="stub/384",
            retrieval=MemoryRetrievalConfig(top_k=3),
        ),
    )
    reviewer = Agent(
        name="compliance_reviewer",
        role="reviewer",
        instructions=(
            "Confirm the disclaimer is present and no diagnostic claims were made. "
            "Approve only if the response is appropriate for internal staff lookup."
        ),
    )
    wf = (
        Workflow("staff_protocol_lookup")
        .step(lookup)
        .step(reviewer, hitl=HitlConfig(approval_key="clinical_disclaimer", timeout_seconds=300))
    )
    result = wf.run("Summarize Protocol A for adult hydration monitoring.")
    print(result.output)


if __name__ == "__main__":
    main()
