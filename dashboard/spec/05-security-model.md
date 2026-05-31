# Security model

## Trust boundaries

```text
Browser (untrusted)
    → BFF (operator session)
        → ArcFlow Server admin API (ARCFLOW_ADMIN_API_KEY)
            → Postgres / Qdrant / Registry

Browser (public site)
    → Relay (site token + Origin header)
        → ArcFlow Server (scoped upstream key)
```

## Secrets handling

| Secret | Where it lives | Never in |
|--------|----------------|----------|
| `ARCFLOW_ADMIN_API_KEY` | BFF env, CI secrets | Browser bundle, git |
| `site_token` | Operator env (`VITE_ARCFLOW_SITE_TOKEN`), Relay DB | Admin GET responses |
| `ARCFLOW_API_KEY` | Server env, Relay upstream config | Frontend |
| LLM provider keys | Server env or Tier 2 vault | Browser |

## Site token lifecycle

1. Created on `POST /v1/admin/sites` or `POST .../tokens/rotate`
2. Shown once in UI with explicit copy warning
3. Stored hashed in Relay; old token invalid after rotate
4. Frontend embeds token in build-time env (static sites) or server-side proxy

## Origin enforcement

Relay checks `Origin` or `Referer` against `allowed_origins` before accepting runs. Dashboard must:

- Require at least one origin before marking site "production ready"
- Warn when origin is `http://localhost` in non-dev contexts

## SEC-1 alignment

Dashboard code and BFF must pass repository SEC-1 checks:

- No hardcoded API keys or tokens
- No logging of `site_token`, admin key, or LLM keys
- Dependencies scanned in CI

## Threat considerations

| Threat | Mitigation |
|--------|------------|
| Admin key leak from browser | BFF pattern; no admin key in Vite bundle |
| Site token scrape from static bundle | Token is public to frontend by design; origin + rate limit bound abuse |
| CSRF on BFF | SameSite cookies, CSRF tokens on mutating BFF routes |
| Token rotation race | UI confirms rotation; shows new token once; old sessions fail closed at Relay |

## Compliance notes

- Knowledge ingest text may contain PII; operators responsible for data policy
- Logs: dashboard should not send ingest body to analytics
