# Level 4: Certified ArcFlow Architect

**Audience:** `[platform]` `[compliance]`

**Title:** Certified ArcFlow Architect

**Prerequisite:** [Level 3: Platform Engineer](level-3-platform-engineer.md)

## What certified means at this level

You understand Rust runtime architecture (`arcflow-core`) and the reasoning behind major decisions, select the correct integration surface for any scenario, design multi-tenant deployments with security boundaries, contribute to or evaluate RCS contract changes, audit deployments for SEC-1 compliance using Postgres queries and trace exports, design enterprise reliability patterns, explain graph join semantics and `max_iterations` guards, compare embedded SDK versus server deployment tradeoffs, reason about deferred features and advise on workarounds, and design production RAG ingestion pipelines.

## Competencies added over Level 3

| Competency | Demonstration |
|------------|---------------|
| Runtime depth | Explain scheduler, trace bridge, provider abstraction |
| Surface selection | Written rationale for SDK vs server vs static vs Relay |
| Multi-tenant design | Tenant isolation for keys, data, and namespaces |
| RCS and contracts | Schema evolution impact analysis |
| SEC-1 audit | Query plus export based audit checklist |
| Reliability at scale | Retry, fallback, recovery, timeout strategy document |
| Graph semantics | Join rules, iteration guards, FP-1.01 limits |
| Deferred features | Workaround for one FP item with accepted tradeoff |
| Production RAG | Ingestion pipeline design with chunking and ops |

## Required reading

All prior level reading plus:

| Topic | Document |
|-------|----------|
| Architecture (deep) | [Architecture overview](../concepts/architecture-overview.md) |
| RCS | [The RCS contract](../concepts/the-rcs-contract.md) |
| Execution model | [Execution model](../concepts/execution-model.md) |
| Maturity | [Maturity and known gaps](../concepts/maturity-and-known-gaps.md) |
| Surfaces | [Surfaces and when to use them](../concepts/surfaces-and-when-to-use-them.md) |
| SEC-1 | [SEC-1 and data safety](../concepts/sec-1-and-data-safety.md), [SEC-1 rules](../guides/observability/sec-1-rules.md) |
| Graph | [Graph workflows](../guides/workflows/graph-workflows.md) |
| RAG ops | [Knowledge ingestion](../guides/memory-and-rag/knowledge-ingestion.md), [vector RAG pipeline](../guides/memory-and-rag/vector-rag-pipeline.md) |
| Reliability | Full [reliability](../guides/reliability/recovery-and-resume.md) and [workflow](../guides/workflows/step-fallbacks.md) guides |
| Trace complete | [Trace event reference](../guides/observability/trace-event-reference.md) |
| Normative contracts | `contracts/normative/` in repository |
| Capabilities appendices | Appendices A through K in capabilities reference (when published) |
| Sprint plans | Implementation plans as architecture reference in repo |

## Tutorial tracks

All tracks A through H should be complete before the architect project. Track H IDE tooling supports design reviews with graph previews.

## Practical project

Design a **multi-tenant enterprise deployment** from scratch. Deliver written artifacts, not only running code.

### Deliverables

| Deliverable | Content |
|-------------|---------|
| Architecture diagram | Server, Relay, Postgres, Qdrant, tenants, admin plane |
| Surface selection rationale | Per integration type: backend job, browser chat, batch ingest, compliance audit |
| Security boundary document | Keys, network zones, data residency, tenant isolation |
| Deployment runbook | Install, migrate, upgrade, rollback, backup |
| SEC-1 audit checklist | Postgres queries and trace sampling procedure |
| Deferred feature recommendation | One of FP-1.01, FP-2, FP-3.01, FP-4, FP-5 with workaround |

### Multi-tenant design requirements

| Area | Decision to document |
|------|---------------------|
| Identity | Separate admin vs runtime vs site tokens per tenant |
| Data | Namespace strategy for vector memory and published workflows |
| Network | Relay per region vs shared Relay with site ids |
| Blast radius | Failure containment if one tenant overloads Qdrant |
| Compliance | Trace retention and export policy per tenant |

### SEC-1 audit checklist (sample)

| Step | Action |
|------|--------|
| 1 | Export 10 random run traces via CLI or HTTP |
| 2 | Search JSON for forbidden fields (prompt, message body, chunk text) |
| 3 | Query Postgres trace tables for column sizes and patterns per schema docs |
| 4 | Review Relay and server log sampling config |
| 5 | Confirm static bundles contain no runtime keys |
| 6 | Document findings and remediation |

### Graph and reliability section

Explain in prose:

| Topic | Must address |
|-------|--------------|
| Join semantics | What `wait_for` guarantees |
| `max_iterations` | Loop guard purpose |
| FP-1.01 | Why not to rely on graph resume in production yet |
