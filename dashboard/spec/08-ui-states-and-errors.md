# UI states and errors

## Global patterns

| State | Visual | Behavior |
|-------|--------|----------|
| Loading | Skeleton rows or spinner on primary button | Disable duplicate submits |
| Empty | Icon + one sentence + single CTA | Link to docs |
| Success | Toast + inline confirmation | Auto-dismiss toast after 5s |
| Error | Banner above form | Preserve user input |

## Per-screen states

### Create site — success

- Full-screen or modal: relay URL, site token, copy buttons
- Warning: "Token will not be shown again"
- Primary: Go to site detail

### Create site — validation error

- Inline field errors for empty name or invalid origin URL
- Server 400 message in banner

### Knowledge ingest — empty text

- Disable submit; inline "Add text or upload a file"

### Knowledge ingest — success

- Show `chunks_ingested` and namespace
- Optional: link to test chat

### Chat publish — in progress

- Button loading state; disable instructions edit

### Chat publish — success

- Badge: `chat@1.0.0` (name + version)
- Show `schema_hash` truncated with copy

### Rotate token — confirm

- Modal: "This invalidates the current token immediately"
- Require typing site display name or checkbox confirm

### Rotate token — success

- Same one-time token UX as create site

## HTTP error mapping

| Status | User message | Action |
|--------|--------------|--------|
| 401 | Check admin session or API key configuration | Link to BFF env docs |
| 403 | This key cannot access admin routes | Contact operator |
| 404 | Site not found | Return to sites list |
| 429 | Too many requests | Retry after delay |
| 500 | ArcFlow server error | Show request id if present; retry |

## Accessibility

- WCAG 2.1 AA contrast on controls
- Focus trap in rotate and create success modals
- Keyboard: tab order through create-site flow
- Mobile: stack fields; sticky Save on Chat tab

## Analytics (optional)

Track funnel events without PII: `site_created`, `knowledge_ingested`, `chat_published`, `token_rotated`. Do not send ingest body or tokens.
