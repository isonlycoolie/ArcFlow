
# Webhook HMAC

Security reference for ArcFlow external callback verification. External integrators POST outcomes to `POST /v1/runs/{run_id}/external/{binding_id}`. Treat this endpoint like a payment webhook: authenticate with Bearer API key, verify HMAC on raw body bytes, reject invalid signatures before parsing sensitive fields.

Tutorial: [Webhook security guide](../guides/external-integrations/webhook-security.md).

Verification runs in the core external module and the server external handler. See [Webhook security](../guides/external-integrations/webhook-security.md).

## Required configuration

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_SERVER_API_KEY` | Bearer auth on POST |
| `ARCFLOW_WEBHOOK_SECRET` | HMAC shared secret |
| `ARCFLOW_POSTGRESQL_URL` | Persist run state for external bindings |

Without webhook secret, handler returns error requiring configuration.

## Required headers

| Header | Purpose |
|--------|---------|
| `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>` | Runtime authentication |
| `X-ArcFlow-Signature` | HMAC-SHA256 over exact request body bytes |
| `Content-Type: application/json` | Outcome JSON |
| `X-Idempotency-Key` | Optional duplicate suppression |

## HMAC computation (ArcFlow signing outbound or integrator signing inbound)

1. Read the exact POST body as bytes (no re-serialization after parse).
2. Compute HMAC-SHA256 with `ARCFLOW_WEBHOOK_SECRET`.
3. Encode digest as lowercase hex.
4. Send header `X-ArcFlow-Signature: sha256=<hex>`.

Header may also be raw hex without `sha256=` prefix; server accepts both.

### Python (integrator)

```python
import hashlib
import hmac
import json

def sign_body(secret: str, body: bytes) -> str:
    digest = hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()
    return f"sha256={digest}"

outcome = {
    "binding_id": "payment_webhook",
    "status": "success",
    "fields": {"transaction_id": "tx_123"},
}
body = json.dumps(outcome).encode()
signature = sign_body("your-webhook-secret", body)
```

### TypeScript (integrator)

```typescript
import { createHmac } from "node:crypto";

function signBody(secret: string, body: Buffer): string {
  const digest = createHmac("sha256", secret).update(body).digest("hex");
  return `sha256=${digest}`;
}

const outcome = { binding_id: "payment_webhook", status: "success" };
const body = Buffer.from(JSON.stringify(outcome));
const signature = signBody(process.env.ARCFLOW_WEBHOOK_SECRET!, body);
```

### Verifying at a gateway (optional)

```python
import hashlib
import hmac

def verify_webhook_signature(secret: str, body: bytes, signature_header: str) -> bool:
    expected = hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()
    provided = signature_header.strip().removeprefix("sha256=")
    if len(provided) != len(expected):
        return False
    return hmac.compare_digest(provided, expected)
```

## Constant-time comparison

Server uses constant-time digest comparison (`subtle::ConstantTimeEq` in Rust) to reduce timing side channels. Integrators should use `hmac.compare_digest` (Python) or `timingSafeEqual` (Node).

Reject signatures whose hex length does not match expected digest length before comparison.

## Secret rotation with dual-verify window

ArcFlow R1 supports a single active `ARCFLOW_WEBHOOK_SECRET`. For rotation without dropping callbacks:

1. Deploy server accepting **both** old and new secret (requires custom gateway dual-verify or brief maintenance window).
2. Update integrators to sign with new secret.
3. Switch server env to new secret only.
4. Revoke old secret.

Standard single-secret rotation: pause integrator traffic, update secret, resume (expect brief 401/422 on mismatched signatures).

## Failure behavior

| Condition | HTTP | Notes |
|-----------|------|-------|
| Missing Bearer | 401 | |
| Bad HMAC | 401 or 422 | Signature rejected before outcome persist |
| Missing webhook secret on server | 503 | Configuration error |
| Duplicate idempotency key | 200 with `already_processed` | Safe retry |

Failed verification must not parse or persist outcome fields.

## curl example

```bash
BODY='{"binding_id":"payment_webhook","status":"success"}'
SIG=$(printf '%s' "$BODY" | openssl dgst -sha256 -hmac "$ARCFLOW_WEBHOOK_SECRET" | awk '{print $2}')

curl -X POST "http://localhost:8080/v1/runs/$RUN_ID/external/payment_webhook" \
  -H "Authorization: Bearer $ARCFLOW_SERVER_API_KEY" \
  -H "Content-Type: application/json" \
  -H "X-ArcFlow-Signature: sha256=$SIG" \
  -d "$BODY"
```

## Network reachability

Integrators must reach `arcflow-server` over HTTPS in production. Do not embed webhook secrets in browser JavaScript; post from backend workers.

## Related pages

- [External callbacks](../guides/external-integrations/external-callbacks.md)
- [SEC-1 compliance](sec-1-compliance.md)
- [Self-hosted security](self-hosted-security.md)
