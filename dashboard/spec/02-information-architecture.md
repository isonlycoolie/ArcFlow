# Information architecture

## Top-level navigation

```text
Dashboard
├── Sites (home)
│   ├── Create site
│   └── Site detail
│       ├── Overview
│       ├── Chat (default tab)
│       ├── Knowledge
│       ├── Workflows
│       ├── Keys and Deploy
│       └── Usage (Tier 2)
├── Account / Billing (Tier 2)
└── Docs (external link)
```

## Screen responsibilities

### Sites list

- Table: display name, site id, allowed origins count, created date
- Primary CTA: Create site
- Empty state: illustration, copy, link to quickstart

### Create site

Fields map to `POST /v1/admin/sites`. Success screen shows relay URL and site token **once** with copy buttons.

### Site detail — Overview

Read-only metadata from `GET /v1/admin/sites/{id}`. Actions: rotate token, edit origins (via Keys tab or PATCH).

### Chat tab (default)

- Instructions textarea → `chat_instructions` via PATCH or publish body
- Publish button → `POST .../workflows/chat/publish`
- After publish: show workflow name + semver version

### Knowledge tab

- Drag-drop or paste text → `POST .../knowledge/ingest`
- Show `chunks_ingested` and namespace on success
- Link to RAG upload guide for content structure

### Workflows tab

- List published versions for `default_workflow_name` (registry read; builder Tier 2)

### Keys and Deploy

- Env snippet for frontend (`VITE_*` vars)
- Allowed origins editor → PATCH
- Rotate token → confirm modal → `POST .../tokens/rotate`

### Usage (Tier 2)

- Chart from `arcflow_site_usage_daily` when meter exists

## Default operator flow

1. Create site → copy env vars
2. Knowledge → ingest FAQs/docs
3. Chat → write instructions → Save and publish
4. Keys and Deploy → verify origins match frontend URL
