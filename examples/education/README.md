# CS401 Course Assistant

## Problem

A mid-size university runs **CS401: Applied Machine Learning** with 180 enrolled students. During the first two weeks of each term, instructors and TAs spend hours answering the same questions:

- When is the midterm?
- What is the late policy for Problem Set 2?
- Is the final project individual or team-based?

Email and Slack threads duplicate content that already exists in the official syllabus and policy addendum. Students in other time zones wait until office hours for answers that should be instant.

## Who this is for

| Role | Goal |
|------|------|
| **Course admin** | Deflect repetitive syllabus questions without building a custom chatbot |
| **Platform engineer** | Prove RAG over a isolated namespace per course (`course-401-fall2026`) |
| **Compliance** | Keep answers grounded in ingested documents only |

## What ArcFlow demonstrates

- **Vector memory** scoped to a course namespace
- **Ingest** of syllabus text before the workflow runs
- **Single-agent** Q&A workflow with retrieval (`top_k=3`)
- **Stub embedding** for local runs; swap to `openai/text-embedding-3-small` in production

## Prerequisites

- Python SDK: `pip install -e sdk-python`
- No Qdrant required for this demo (in-memory vector store)
- Optional: set `OPENAI_API_KEY` if you replace stub with a live provider

## Run

```bash
python examples/education/syllabus_assistant.py
```

Optional custom question:

```bash
python examples/education/syllabus_assistant.py "Can I use a late day on PS3?"
```

## Verify

- Output references **Week 6 midterm**, **48-hour late policy**, or **team final project** from [`data/syllabus_cs401.md`](data/syllabus_cs401.md)
- Trace (if exported) includes `MemoryRetrieved` with `chunk_count > 0`
- Agent does not invent dates or policies not present in the syllabus

## Production notes

- Ingest PDFs/syllabi via admin API or batch job; one namespace per course offering
- Add HITL if answers may affect grades or accommodations (see [healthcare/](../healthcare/) pattern)
- Static student portal: publish a `chat` workflow via [static/chat-rag/](../static/chat-rag/) + Relay

## Files

| File | Purpose |
|------|---------|
| [`syllabus_assistant.py`](syllabus_assistant.py) | Ingest + workflow entrypoint |
| [`data/syllabus_cs401.md`](data/syllabus_cs401.md) | Sample syllabus source document |
