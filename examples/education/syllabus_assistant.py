# CS401 syllabus assistant — vector RAG over an isolated course namespace.

from __future__ import annotations

import sys
from pathlib import Path

from arcflow import Agent, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, VectorStore, Workflow

COURSE_NAMESPACE = "course-401-fall2026"
SYLLABUS_PATH = Path(__file__).parent / "data" / "syllabus_cs401.md"
DEFAULT_QUESTION = "When is the midterm and what is the late policy for problem sets?"


def load_syllabus() -> str:
    return SYLLABUS_PATH.read_text(encoding="utf-8")


def main() -> None:
    question = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_QUESTION
    syllabus = load_syllabus()

    store = VectorStore()
    store.ingest(COURSE_NAMESPACE, "syllabus", syllabus)

    memory = MemoryConfig(
        MemoryType.VECTOR,
        MemoryScope.AGENT,
        namespace=COURSE_NAMESPACE,
        embedding="stub/384",
        retrieval=MemoryRetrievalConfig(mode="dense", top_k=3),
    )
    tutor = Agent(
        name="course_tutor",
        role="teaching_assistant",
        instructions=(
            "You are the CS401 course assistant. Answer only from retrieved syllabus content. "
            "If the syllabus does not contain the answer, say you cannot find it and "
            "direct the student to post in #cs401-questions or attend TA hours. "
            "Never invent dates, policies, or grades."
        ),
        memory=memory,
    )
    wf = Workflow("cs401_syllabus_assistant").step(tutor)
    result = wf.run(question)
    print(result.output)


if __name__ == "__main__":
    main()
