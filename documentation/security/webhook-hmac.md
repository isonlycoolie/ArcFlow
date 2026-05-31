**Audience:** `[platform]` `[compliance]`

# Webhook HMAC

Security reference for ArcFlow external callback verification. External integrators POST outcomes to `POST /v1/runs/{run_id}/external/{binding_id}`. Treat this endpoint like a payment webhook: authenticate with Bearer API key, verify HMAC on raw body bytes, reject invalid signatures before parsing sensitive fields.

Tutorial: [Webhook security guide](../guides/external-integrations/webhook-security.md).

Implementation: `runtime/arcflow-core/src/external/webhook.rs`, `server/arcflow-server/src/handlers/external.rs`.

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

