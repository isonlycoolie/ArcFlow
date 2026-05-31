"""Course Q&A with RAG over a syllabus namespace (Phase 2-Pro)."""

from __future__ import annotations

from arcflow import Agent, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, Workflow
from arcflow.memory import VectorStore

SYLLABUS = """Week 1: Introduction to ML.
Week 2: Retrieval-augmented generation for course assistants.
Week 3: Evaluation and safety."""


def main() -> None:
    store = VectorStore()
    store.ingest("course-101", "syllabus", SYLLABUS)
    memory = MemoryConfig(
        MemoryType.VECTOR,
        MemoryScope.AGENT,
        namespace="course-101",
        embedding="stub/384",
        retrieval=MemoryRetrievalConfig(mode="dense", top_k=3),
    )
    tutor = Agent(
        name="tutor",
        role="tutor",
        instructions="Answer student questions using retrieved syllabus chunks.",
        memory=memory,
    )
    wf = Workflow("course_qa").step(tutor)
    print(wf.run("What is covered in week 2?").output)


if __name__ == "__main__":
    main()
