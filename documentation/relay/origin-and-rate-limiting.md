
# Origin and rate limiting

Relay enforces two site-level controls before proxying to `arcflow-server`: **Origin allowlist** and **requests per minute**. Both are configured when a site is created or patched via admin API, or inline in `ARCFLOW_RELAY_SITES_JSON` for BYO Relay.

## allowed_origins

Each site stores `allowed_origins` as a string array (e.g. `["https://www.acme.com", "https://app.acme.com"]`).

On each browser request, Relay reads the `Origin` header (or `Referer` fallback in some paths) and compares against the list. Mismatch returns **403**:

```json
{
 "error": {
 "code": "origin_not_allowed",
 "message": "[ArcFlow Relay] Origin not allowed for this site."
 }
}
```

### Setting origins at create

```bash
curl -s -X POST http://localhost:8080/v1/admin/sites \
 -H "Authorization: Bearer dev-admin" \
 -H "Content-Type: application/json" \
 -d '{
 "display_name": "Acme",
 "allowed_origins": ["https://www.acme.com"],
 "rate_limit_rpm": 60,
 "allow_inline": false
 }'
```

### Updating origins

```bash
curl -s -X PATCH "http://localhost:8080/v1/admin/sites/site_abc123" \
 -H "Authorization: Bearer dev-admin" \
 -H "Content-Type: application/json" \
 -d '{"allowed_origins":["https://www.acme.com","https://staging.acme.com"]}'
```

Redeploy or reload Relay if using JSON file config so in-memory site store picks up changes.

## Development vs production

| Environment | Typical Origin |
|-------------|----------------|
| Vite dev server | `http://localhost:5173` |
| Preview deploy | `https://preview-xyz.vercel.app` |
| Production | Exact production origin, no wildcard |

Relay does not support `*` origin in production configurations. Add each deploy preview origin explicitly or use separate dev sites.

### Local test with curl

Relay expects a browser-like Origin:

```bash
curl -s -X POST "http://localhost:8090/v1/sites/s_dev/runs" \
 -H "Authorization: Bearer st_live_devtoken" \
 -H "Origin: http://localhost:5173" \
 -H "Content-Type: application/json" \
 -d '{"workflow_ref":{"name":"chat","version":"^1.0.0"},"input":"test"}'
```

Wrong origin:

```bash
curl -s -o /dev/null -w "%{http_code}\n" \
 -X POST "http://localhost:8090/v1/sites/s_dev/runs" \
 -H "Authorization: Bearer st_live_devtoken" \
 -H "Origin: https://evil.example" \
 -H "Content-Type: application/json" \
 -d '{"workflow_ref":{"name":"chat","version":"^1.0.0"},"input":"test"}'
```

Expect **403**.

## rate_limit_rpm

`rate_limit_rpm` caps create + poll traffic per site per minute (implementation uses in-memory token bucket keyed by site id). Default **60** at site create.

When exceeded, Relay returns **429**:

```json
{
 "error": {
 "code": "rate_limited",
 "message": "[ArcFlow Relay] Rate limit exceeded."
 }
}
```

### Tuning guidance

| Site type | Suggested RPM |
|-----------|---------------|
| Internal demo | 60 |
| Customer support widget | 120 to 300 |
| High-traffic marketing page | Scale Relay replicas; raise RPM with monitoring |

Rate limits protect upstream LLM cost and server capacity. They do not replace WAF or bot management at the CDN edge.

## Security implications

- **Wrong Origin config** is the most common production misconfiguration. Symptom: browser CORS-like failures with 403 from Relay.
- **Leaked site token** plus correct Origin still allows abuse within RPM. Rotate token immediately via admin rotate endpoint.
- Site tokens are not LLM keys, but they still authorize workflow runs billed to your infrastructure.

## Related pages

- [Site lifecycle](../static-product/site-lifecycle.md)
- [Security model](../static-product/security-model.md)
- [BYO Relay deployment](byo-relay-deployment.md)
