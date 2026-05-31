# Level 3: Certified ArcFlow Platform Engineer


**Title:** Certified ArcFlow Platform Engineer

**Prerequisite:** [Level 2: Systems Engineer](level-2-systems-engineer.md)

## What certified means at this level

You deploy `arcflow-server` and `arcflow-relay` in production-oriented configurations, manage Postgres migrations, configure authentication tiers correctly, operate the static product (sites, knowledge, publish, Relay), monitor health and readiness, rotate API keys without downtime, configure OpenTelemetry export, apply the production checklist, and enforce SEC-1 compliance in running systems.

## Competencies added over Level 2

| Competency | Demonstration |
|------------|---------------|
| Deployment | Docker Compose or K8s manifests for server, Relay, Postgres, Qdrant |
| Migrations | Apply and re-run idempotently; schema at head |
| Auth tiers | Runtime key, admin key, scoped site tokens used correctly |
| Static product | Site provision, ingest, publish, browser chat via Relay |
| Observability | `/health`, `/ready`, persisted traces, optional OTel |
| Key rotation | Rotate runtime or site token without failed requests |
| SEC-1 | Traces and exports contain metadata only |
| Production checklist | Complete operator checklist for your environment |

## Required reading

Read server, Relay, static product, deployment, operator, and security documentation in your checkout (organization plan lists Server all, Relay all, Static Product all, Deployment all, Operator all, Security all). Core guides already in `documentation/`:

| Topic | Document |
|-------|----------|
| Traces and SEC-1 | [Execution traces](../guides/observability/execution-traces.md), [SEC-1 rules](../guides/observability/sec-1-rules.md) |
| OpenTelemetry | [OpenTelemetry](../guides/observability/opentelemetry.md) |
| Data safety | [SEC-1 and data safety](../concepts/sec-1-and-data-safety.md) |
| Surfaces | [Surfaces and when to use them](../concepts/surfaces-and-when-to-use-them.md) |

## Tutorial tracks

| Track | Topic |
|-------|-------|
| [F](../tutorials/track-f-static-product.md) | Static product and origin enforcement |
| [G](../tutorials/track-g-operations.md) | Migrations, readiness, CLI trace |

Also complete example walkthroughs:

| Example | Link |
|---------|------|
| Static chat widget | [static-chat-widget](../examples/static-chat-widget.md) |
| Relay BYO | [relay-byo-deployment](../examples/relay-byo-deployment.md) |

## Practical project

Deploy the **full ArcFlow stack** (server, Relay, Postgres, Qdrant), provision a static product site with knowledge and published workflow, verify SEC-1 compliance, and perform token rotation without downtime.

### Stack components

| Component | Responsibility |
|-----------|----------------|
| Postgres | Runs, registry, migrations |
| Qdrant | Vector memory for site knowledge |
| arcflow-server | Execution and admin APIs |
| arcflow-relay | Browser proxy with Origin checks |
| Static frontend | [`chat-rag`](../examples/static-chat-widget.md) or equivalent |

### Requirements

| Requirement | Detail |
|-------------|--------|
| Production compose or K8s | Not default dev secrets in final config |
| Migrations | Document initial and re-run migrate |
| Static site | Provision, ingest, publish `chat` workflow |
| Browser verification | Chat works from allowed origin; blocked from disallowed |
| SEC-1 audit | Sample trace and HTTP export reviewed; no prompt bodies |
| Key rotation | Procedure documented; demo rotation of runtime or site token |
| OpenTelemetry | Exporter configured or documented skip with reason |
| Runbook | Start, stop, backup, restore outline |

### Suggested commands

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
bash scripts/static-provision-site.sh
bash scripts/static-ingest-knowledge.sh
bash scripts/static-publish-chat.sh
cd examples/static/chat-rag && npm install && npm run dev
curl -sf http://localhost:8080/ready
cargo run -p arcflow-cli -- migrate up
cargo run -p arcflow-cli -- trace RUN_ID --format json
```

BYO Relay optional second phase per [relay-byo-deployment](../examples/relay-byo-deployment.md).

### SEC-1 verification

| Check | Method |
|-------|--------|
| Trace export | CLI or GET trace; confirm metadata fields only |
| Logs | No full user messages at info level |
| Static bundle | No server runtime key in client JS |
| Relay | Site token only in frontend env |

### Key rotation without downtime

Document ordered steps:

1. Issue new runtime key or site token alongside old
2. Update Relay site JSON or server env on rolling instance
3. Deploy frontend new site token if rotated
4. Drain old key after traffic confirms success
5. Revoke old key

Demonstrate at least one rotation in dev with zero failed chat requests during rolling update.

### Pass criteria checklist

| Check | Pass |
|-------|------|
| `/ready` 200 at head | yes |
| Idempotent migrate | yes |
| Static chat end-to-end | yes |
| Origin enforcement | yes |
| SEC-1 sample audit signed off | yes |
| Rotation procedure executed | yes |
| OTel or documented exception | yes |

## Self-assessment checklist

| Question | Answer must be yes |
|----------|-------------------|
| Can you explain three auth tiers and which routes use each? | |
| Can you restore Postgres without replaying user prompts from traces? | |
| Can you operate Relay rate limits per site? | |
| Can you block a compromised site token in minutes? | |

## Next level

Proceed to [Level 4: Architect](level-4-certified-arcflow-architect.md) for multi-tenant design, RCS evolution, and enterprise tradeoffs.
