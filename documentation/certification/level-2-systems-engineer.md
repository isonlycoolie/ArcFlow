# Level 2: Certified ArcFlow Systems Engineer


**Title:** Certified ArcFlow Systems Engineer

**Prerequisite:** [Level 1: Workflow Developer](level-1-workflow-developer.md)

## What certified means at this level

You design graph workflows with conditional routing and joins, operate vector RAG ingest and retrieval, apply retry timeout and fallback patterns, implement HITL approval flows, integrate external callbacks with HMAC verification, configure SDK streaming, and read complex traces including retry events, `MemoryRetrieved`, and HITL states.

## Competencies added over Level 1

| Competency | Demonstration |
|------------|---------------|
| Graph DAG | Branch selection and join nodes with trace proof |
| Vector RAG | Ingest plus query with `MemoryRetrieved` |
| Reliability | Retry, timeout, or step fallback on at least one step |
| HITL | Interrupt and approve via API |
| External callbacks | Outcome posted with verified auth |
| Streaming | `run_stream` or `runStream` consumer |
| Complex traces | Interpret retry and interrupt metadata |

## Required reading

| Topic | Document |
|-------|----------|
| Graph | [Graph workflows](../guides/workflows/graph-workflows.md) |
| RAG | [Vector RAG pipeline](../guides/memory-and-rag/vector-rag-pipeline.md) |
| Retry | [Retry and backoff](../guides/reliability/retry-and-backoff.md) |
| Timeouts | [Timeouts](../guides/reliability/timeouts.md) |
| Fallbacks | [Step fallbacks](../guides/workflows/step-fallbacks.md) |
| Recovery | [Recovery and resume](../guides/reliability/recovery-and-resume.md) |
| HITL | [HITL overview](../guides/human-in-the-loop/hitl-overview.md), [configuring interrupts](../guides/human-in-the-loop/configuring-interrupts.md), [approve and reject](../guides/human-in-the-loop/approve-and-reject.md) |
| External | [External callbacks](../guides/external-integrations/external-callbacks.md), [webhook security](../guides/external-integrations/webhook-security.md) |
| Streaming | [SDK streaming](../guides/streaming/sdk-streaming.md) |
| Trace depth | [Trace event reference](../guides/observability/trace-event-reference.md) |

## Tutorial tracks

Complete with verification checklists:

| Track | Topic |
|-------|-------|
| [B](../tutorials/track-b-server-api.md) | Server API |
| [C](../tutorials/track-c-rag.md) | RAG |
| [D](../tutorials/track-d-graph-workflows.md) | Graph |
| [E](../tutorials/track-e-hitl-and-external.md) | HITL and external |

## Practical project

Build a **graph-routed customer support workflow** with RAG, HITL escalation, retry configuration, and external webhook notification.

### Architecture sketch

```
classify (graph entry)
 ├─ billing branch
 └─ technical branch
 └─ join → answer with RAG
escalation branch → HITL manager approve → external notify webhook
```

### Requirements

| Requirement | Detail |
|-------------|--------|
| Graph routing | At least two conditional branches plus one join |
| RAG | Knowledge base ingested; answers cite retrieved context in output behavior |
| HITL | Escalation path interrupts; approve via `POST.../approve/{key}` |
| Retry | Configure retry on a flaky tool or stub failure injection |
| External webhook | Post outcome via `report_outcome` or signed HTTP |
| Server runtime | Use `arcflow-server` for HITL and external paths |
| Trace pack | Export trace showing `MemoryRetrieved`, graph nodes, interrupt, completion |

### Suggested examples to compose

| Pattern | Example |
|---------|---------|
| Graph join | [`parallel_search.py`](../examples/graph-routing.md) |
| RAG | [`document_qa.py`](../examples/rag-chatbot.md) |
| HITL | [`expense_approval.py`](../examples/hitl-approval-flow.md) |
| External | [`playwright_stub_callback.py`](../examples/external-webhook.md) |
| Streaming (optional UX) | [`chat_stream.py`](../examples/streaming-responses.md) |

### Verification commands

```bash
docker compose -f docker/docker-compose.server.yml up -d
python your_support_workflow/run.py
bash examples/hitl/approve_cli.sh RUN_ID # when escalated
python examples/external/playwright_stub_callback.py --run-id RUN_ID --status success
curl -s "http://localhost:8080/v1/runs/RUN_ID/trace" -H "Authorization: Bearer dev-secret"
```

Pass criteria:

| Check | Pass |
|-------|------|
| Wrong branch not executed | Graph trace proves branch taken |
| `MemoryRetrieved` on FAQ path | yes |
| HITL `Interrupted` then `Completed` after approve | yes |
| External callback accepted | run advances |
| Retry event in trace when failure injected | yes |
| Trace policy | no prompts in exported trace |

## Known gaps to acknowledge

| ID | Impact on project |
|----|-------------------|
| Graph recovery resume | Graph checkpoint resume partial; design linear recovery fallback |
| Server SSE streaming | Server SSE streaming deferred; use SDK streaming or poll |

Note these in your README.

## Self-assessment checklist

| Question | Answer must be yes |
|----------|-------------------|
| Can you explain join preconditions? | |
| Can you rotate HITL approval keys conceptually? | |
| Can you verify webhook HMAC without disabling checks? | |
| Can you read `MemoryRetrieved` without seeing chunk text? | |

## Next level

Proceed to [Level 3: Platform Engineer](level-3-platform-engineer.md) for deployment, static product, and trace data policy operations.
