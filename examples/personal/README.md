# Solo Creator Blog Pipeline

## Problem

**Jordan** publishes one technical blog post per week on a personal site. Each post currently takes 6 to 8 hours:

1. Research the topic and collect links
2. Write a 1,200-word draft
3. Tune title and meta description for SEO

Jordan wants a **repeatable agent pipeline** that produces a draft + SEO package from a single topic prompt, without giving up final editorial control.

## Who this is for

| Role | Goal |
|------|------|
| **Solo creator** | Shrink research, draft, and SEO cycle |
| **Developer advocate** | Show linear multi-agent handoff |

## What ArcFlow demonstrates

- Three-step **linear** workflow: researcher, writer, SEO
- Context handoff between steps (prior step outputs in prompt)
- No RAG; topic provided at run time

## Prerequisites

- Python SDK installed
- Optional: `OPENAI_API_KEY` for live drafts (stub works for structure demo)

## Run

```bash
python examples/personal/weekly_blog_pipeline.py
```

Custom topic:

```bash
python examples/personal/weekly_blog_pipeline.py "Why local-first agent runtimes matter for privacy"
```

## Verify

- Output includes draft-like content and SEO fields (title/meta suggestions)
- Trace shows three `StepCompleted` events
- `run_id` is printed for CLI trace inspection

## Production notes

- Add HITL before publish step for editorial approval
- Store drafts in persistent memory or export to CMS webhook
- See [graph/content_reflection_loop.py](../graph/content_reflection_loop.py) for quality iteration

## Files

| File | Purpose |
|------|---------|
| [`weekly_blog_pipeline.py`](weekly_blog_pipeline.py) | Research, write, and SEO workflow |
