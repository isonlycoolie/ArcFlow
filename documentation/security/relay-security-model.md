
# Relay security model

Deep security analysis of the ArcFlow Relay pattern for static browser products. Relay keeps LLM provider keys on the server while exposing a site-scoped token to the frontend.

## Threat model summary

| Asset | Trust level | Exposure |
|-------|-------------|----------|
| LLM API keys | High secret | Server only |
| `ARCFLOW_SERVER_API_KEY` | High secret | Server, backend, Relay upstream |
| `ARCFLOW_ADMIN_API_KEY` | High secret | BFF/CI only |
| Site token | Public to frontend | Bounded by Relay policy |
| User chat input | Untrusted | Validated server-side |

Primary goal: prevent LLM key exfiltration via browser bundles or DevTools.

## Trust boundaries

```text
Browser (untrusted)
 → Relay (site token + Origin header)
 → ArcFlow Server (scoped upstream runtime key)
 → Postgres / Qdrant / LLM providers

Operator browser (untrusted)
 → BFF (operator session)
 → Admin API (ARCFLOW_ADMIN_API_KEY)
```

See [Static product security model](../static-product/security-model.md).

## Site token scope

Site tokens authenticate Relay routes only. They do not grant:

- Admin API access
- Arbitrary workflow definitions (when `allow_inline: false`)
- LLM provider key retrieval

Upstream server calls use a scoped runtime key limiting callable workflows (typically published `chat` workflow).

## Origin enforcement

Relay middleware checks `Origin` or `Referer` against site `allowed_origins` before accepting runs.

| Scenario | Result |
|----------|--------|
| Allowed production HTTPS origin | Run proxied |
| Missing/wrong origin | Rejected |
| Attacker with scraped token from other origin | Rejected (token alone insufficient) |

Operators must PATCH origins when frontend URL changes.

## Rate limiting

Per-site `rate_limit_rpm` returns **429** when exceeded. Limits abuse from leaked tokens or scripted traffic. Does not replace CDN DDoS protection.

Monitor 429 and origin failure metrics in Relay logs.

## What an attacker can do with a compromised site token

| Action | Possible? |
|--------|-----------|
| Call allowed workflows via Relay from allowed origin | Yes (by design for real users) |
| Call admin routes | No |
| Read LLM keys | No |
| Inline arbitrary workflow JSON | No when `allow_inline: false` |
| Exfiltrate other sites' data | No (site scoped) |
| Run from arbitrary origin | No (origin check) |

## Why `mode: "direct"` is insecure for production

Static SDK `mode: "direct"` embeds `ARCFLOW_SERVER_API_KEY` in the browser. Any visitor can extract the key and call `/v1/runs` directly, bypassing site workflow allowlists and rate limits.

| mode | Production use |
|------|----------------|
| `relay` | Yes |
| `bff` | Yes (your backend holds keys) |
| `direct` | Local dev only |

Example secure config:

```typescript
await runPublished("chat", "^1.0.0", message, {
 mode: "relay",
 relayUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
 siteId: import.meta.env.VITE_ARCFLOW_SITE_ID,
 siteToken: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
});
```

## Trace data policy and browser trace poll

Relay exposes `GET.../trace` to the browser. Trace events are metadata-only trace only. See [Trace data policy compliance](sec-1-compliance.md).

## Token rotation and race conditions

After `POST.../tokens/rotate`, old tokens fail immediately at Relay. Plan deploys to avoid user-visible auth errors during rotation. See [Token rotation](../operator/token-rotation.md).

## CSRF on operator BFF

Dashboard BFF mutating routes should use SameSite cookies and CSRF tokens. Admin key never reaches the browser.

## Related pages

- [Relay deployment](../deployment/relay-deployment.md)
- [API key management](api-key-management.md)
- [Streaming in the browser](../guides/streaming/streaming-in-the-browser.md)
