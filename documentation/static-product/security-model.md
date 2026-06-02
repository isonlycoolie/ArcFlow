
# Static product security model

The static product is designed so **LLM provider keys and master server keys never ship to browsers**. What appears in frontend code is a **site token** scoped to one site, plus public Relay URL. Defense in depth uses Origin allowlists, rate limits, and server-side `allow_inline: false`.

## Threat model summary

| Asset | Browser exposure | Mitigation |
|-------|------------------|------------|
| OpenAI/Anthropic/Gemini keys | Must never appear | Server-side providers only |
| `ARCFLOW_SERVER_API_KEY` | Must never appear in prod | Relay + scoped upstream keys |
| `ARCFLOW_ADMIN_API_KEY` | Never | Operator backend only |
| Site token | Yes (build-time env) | Origin + RPM + rotation |
| Published workflow semver | Yes | No inline agent override when disabled |

## allow_inline: false

When `allow_inline` is false on a site (recommended production default), browsers cannot POST full inline `workflow` + `agents` bodies through Relay. They must use `workflow_ref` via `runPublished`.

Server validation: `validate_static_payload` in run handler rejects dangerous inline overrides for static integrations.

Create site with inline disabled:

```bash
curl -s -X POST http://localhost:8080/v1/admin/sites \
 -H "Authorization: Bearer dev-admin" \
 -H "Content-Type: application/json" \
 -d '{"display_name":"Acme","allowed_origins":["https://www.acme.com"],"rate_limit_rpm":60,"allow_inline":false}'
```

## Origin enforcement

Relay rejects requests whose `Origin` is not in `allowed_origins`. This limits token replay from arbitrary hosts even if the token is copied.

Misconfiguration symptom: **403 origin not allowed** in browser network tab. Fix PATCH on site origins, not CORS on LLM providers.

Detail: [relay/origin-and-rate-limiting.md](../relay/origin-and-rate-limiting.md).

## Site token vs API key

| Token type | Capability |
|------------|------------|
| Site token | Create/poll runs for one site via Relay |
| Scoped runtime key | Upstream server auth (Relay only, not browser) |
| Master server key | Full runtime API (backends only) |
| Admin key | Sites, ingest, publish |

Site tokens cannot call admin routes or arbitrary registry publish.

## direct mode (development only)

`ArcFlowClient` supports `mode: "direct"` against `arcflow-server` with `ARCFLOW_SERVER_API_KEY`. This is for local debugging without Relay.

**Do not use direct mode in production builds.** Any user can extract the key from bundled JavaScript.

For production, use `mode: "relay"` or `mode: "bff"` where your backend holds secrets.

## Server CORS (direct mode)

`ARCFLOW_CORS_ORIGINS` on the server allows browser-direct calls during development. Production static sites should not rely on server CORS; use Relay.

## trace data policy traces

Traces exported through Relay contain metadata only (event kinds, counts, durations). No prompts, tool payloads, or KB chunk text. Compliance reviewers should verify client-side logging does not duplicate user messages into analytics pipelines with PII policies.

See [guides/observability/sec-1-rules.md](../guides/observability/sec-1-rules.md).

## Token rotation

Rotate on leak, employee offboarding, or periodic policy:

```bash
curl -s -X POST "http://localhost:8080/v1/admin/sites/site_abc123/tokens/rotate" \
 -H "Authorization: Bearer dev-admin"
```

Redeploy frontend env with new token before announcing rotation to avoid user-facing outages.

## Server SSE (streaming deferred)

Do not expose SSE endpoints to browsers until server SSE streaming ships. Current polling model does not widen the attack surface with long-lived authenticated streams.

## Related pages

- [static-product/overview.md](overview.md)
- [server/authentication.md](../server/authentication.md)
- Dashboard spec [05-security-model.md](../static-product/security-model.md)
