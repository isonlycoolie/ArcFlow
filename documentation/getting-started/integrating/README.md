# Integrating ArcFlow

**Audience:** `[developer]` `[platform]`

## What this track is

The integrating track is for teams that have run a local stub workflow and now need to choose how ArcFlow fits their stack. The lessons compare embedded SDK runs with server-backed HTTP execution, walk through a first server API call, and introduce the two pause patterns that require Postgres: external callbacks and human-in-the-loop (HITL) approval.

Nothing here replaces the full guides under `documentation/guides/`. These pages are short introductions with one runnable example each and pointers to normative detail.

## How to use it

Read the lessons in order if you are deciding deployment shape for the first time. Skip lesson 1 if you already know you need the HTTP server. Each page includes a **Verify** section so you can confirm behavior before moving on.

| Order | Lesson | What you learn |
|-------|--------|----------------|
| 1 | [Embedded SDK vs server](01-embedded-sdk-vs-server.md) | When to embed the runtime vs call `arcflow-server` |
| 2 | [Server API first run](02-server-api-first-run.md) | Docker stack, `POST /v1/runs`, poll, trace |
| 3 | [External callbacks intro](03-external-callbacks-intro.md) | `ExternalOutcome.report` / `report_outcome` lifecycle |
| 4 | [HITL approval intro](04-hitl-approval-intro.md) | Interrupt, approve, resume on the server |

## Before you start

Complete at least one path from [Getting started paths](../paths/README.md):

| Path | Document |
|------|----------|
| Fastest local run | [First workflow in five minutes](../first-workflow-in-five-minutes.md) |
| Python SDK detail | [Python quickstart](../quickstart-python.md) |
| HTTP without SDK | [Server API quickstart](../quickstart-server-api.md) |

For server lessons (2 through 4), you need Docker with Compose v2 and ports 8080 and 5432 available on your machine.

## After this track

| Goal | Document |
|------|----------|
| Full external callback guide | [External callbacks](../../guides/external-integrations/external-callbacks.md) |
| Webhook HMAC verification | [Webhook security](../../guides/external-integrations/webhook-security.md) |
| HITL configuration detail | [Configuring interrupts](../../guides/human-in-the-loop/configuring-interrupts.md) |
| HITL approve API | [Approve and reject](../../guides/human-in-the-loop/approve-and-reject.md) |
| Guided server tutorial | [Track B: Server API](../../tutorials/track-b-server-api.md) |
| HITL and external tutorial | [Track E: HITL and external](../../tutorials/track-e-hitl-and-external.md) |
| Static site product | [Static site chatbot](../paths/static-site-chatbot.md) |

**Source:** `documentation/getting-started/paths/`, `documentation/guides/`, `server/arcflow-server/README.md`; capabilities reference §12, §28 Tracks B and E.
