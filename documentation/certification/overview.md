# ArcFlow certification program


ArcFlow certification is a self-assessed competency path from first workflow to certified architect. Each level defines what you must understand, what you must build, and how you verify success. There is no external exam today; you certify yourself when your project meets the stated criteria.

## Philosophy

Certification rewards working systems, not memorized facts. You demonstrate competence by running code, reading traces, operating services, and documenting decisions. If you cannot reproduce the verification steps in the level document, you are not ready to claim that level.

Self-assessment means you own the honesty bar. Share artifacts (repo links, run ids, deployment checklists) with your team or customers as proof, but ArcFlow does not issue signed credentials in the current program.

## Four levels and real-world roles

| Level | Title | Typical job responsibilities |
|-------|-------|------------------------------|
| 1 | [Certified ArcFlow Workflow Developer](level-1-workflow-developer.md) | Builds SDK workflows, configures agents and providers, reads basic traces |
| 2 | [Certified ArcFlow Systems Engineer](level-2-systems-engineer.md) | Adds graph routing, RAG, HITL, external callbacks, streaming, reliability patterns |
| 3 | [Certified ArcFlow Platform Engineer](level-3-platform-engineer.md) | Deploys server and Relay, operates migrations, static product, trace data policy compliance |
| 4 | [Certified ArcFlow Architect](level-4-certified-arcflow-architect.md) | Designs multi-tenant deployments, evaluates surfaces, audits compliance, guides schema evolution |

Levels are cumulative. Level 2 assumes Level 1 competencies; Level 4 assumes all prior levels.

## Competency progression

| Capability area | L1 | L2 | L3 | L4 |
|-----------------|:--:|:--:|:--:|:--:|
| Linear SDK workflows | yes | yes | yes | yes |
| Provider and tool configuration | yes | yes | yes | yes |
| Basic trace reading | yes | yes | yes | yes |
| CLI init and run | yes | yes | yes | yes |
| Server HTTP API | | yes | yes | yes |
| Graph DAG routing and joins | | yes | yes | yes |
| Vector RAG ingest and query | | yes | yes | yes |
| Retry, timeout, fallback | | yes | yes | yes |
| HITL and external callbacks | | yes | yes | yes |
| SDK streaming | | yes | yes | yes |
| Docker or K8s deployment | | | yes | yes |
| Auth tiers and key rotation | | | yes | yes |
| Static product and Relay | | | yes | yes |
| OpenTelemetry export | | | yes | yes |
| trace data policy operational enforcement | | | yes | yes |
| Rust runtime and workflow specification depth | | | | yes |
| Multi-tenant architecture | | | | yes |
| Enterprise reliability design | | | | yes |
| Deferred feature tradeoffs | | | | yes |

## How to use this path

1. Complete required reading for the target level (linked in each level doc).
2. Finish tutorial tracks listed for that level.
3. Build the practical project without skipping verification steps.
4. Compare your artifacts to the level checklist.
5. Claim the level internally when all checks pass.

Recommended track order: A through H in [tutorials](../tutorials/track-a-first-workflow.md), skipping sections you already mastered but not skipping verification commands.

## Practical project standards

Every level project should include:

| Artifact | Purpose |
|----------|---------|
| Runnable code or deployment manifests | Proves execution |
| Trace or HTTP export samples | Proves observability |
| Short README with verify commands | Lets reviewers reproduce |
| Known gaps noted | Shows maturity awareness (FP items) |

Do not submit screenshots alone. Commands and run ids are evidence.

## Relationship to examples and guides

| Resource | Role in certification |
|----------|----------------------|
| [Examples catalog](../examples/catalog.md) | Starting points for level projects |
| [Guides](../guides/workflows/linear-workflows.md) | Depth behind each competency |
| [Concepts](../concepts/what-is-arcflow.md) | Mental model for Level 1 and 4 |
| Tutorial tracks A to H | Structured verification before projects |

## Level documents

| Level | Document |
|-------|----------|
| 1 | [Certified ArcFlow Workflow Developer](level-1-workflow-developer.md) |
| 2 | [Certified ArcFlow Systems Engineer](level-2-systems-engineer.md) |
| 3 | [Certified ArcFlow Platform Engineer](level-3-platform-engineer.md) |
| 4 | [Certified ArcFlow Architect](level-4-certified-arcflow-architect.md) |
