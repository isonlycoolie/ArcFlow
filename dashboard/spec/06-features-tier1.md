# Tier 1 features (R1)

R1 matches Relay R1 exit criteria: one site, chat workflow, knowledge ingest, no inline workflows.

## F1 — Create site

**API:** `POST /v1/admin/sites`

**UI**

- Form: display name, allowed origins (multi-input), optional chat instructions
- Success modal: `relay_url`, `site_token`, env snippet
- Checkbox: "I have saved the site token"

**Acceptance**

- Operator can paste env vars into a Vite static project and reach Relay health

## F2 — Site overview

**API:** `GET /v1/admin/sites/{id}`

**UI**

- Read-only cards: id, namespace, rate limit, created date
- Badge: origins count, inline disabled

## F3 — Knowledge ingest

**API:** `POST .../knowledge/ingest`

**UI**

- Text area or file upload (client reads file → text)
- Optional document key field
- Success toast with chunk count

**Acceptance**

- Follow-up chat run retrieves ingested content (verified via static smoke or manual run)

## F4 — Chat publish

**API:** `PATCH` (instructions) + `POST .../workflows/chat/publish`

**UI**

- Instructions textarea with character hint
- "Save and publish" single action
- Show published version and schema hash

**Acceptance**

- Relay run uses published workflow version

## F5 — Origins and rate limit

**API:** `PATCH /v1/admin/sites/{id}`

**UI**

- List editor for origins
- Numeric input for `rate_limit_rpm` (min 1)

## F6 — Rotate token

**API:** `POST .../tokens/rotate`

**UI**

- Destructive confirm modal
- New token display once
- Reminder to update frontend env and redeploy

## F7 — Deploy snippet

**UI only**

```bash
VITE_ARCFLOW_RELAY_URL=<relay_url>
VITE_ARCFLOW_SITE_TOKEN=<site_token>
```

Link to [examples/static](../../examples/static/README.md) and meta-repo template.

## Explicitly off in Tier 1

- Inline workflow JSON editor
- BYO OpenAI/Anthropic key forms
- Usage charts
- Multi-site billing
