**Audience:** `[platform]` `[compliance]`

# Webhook security for external callbacks

External callbacks arrive on `POST /v1/runs/{run_id}/external/{binding_id}`. Treat this endpoint like a payment webhook: authenticate with Bearer API key, verify HMAC on the raw body, reject replay where possible, and never log sensitive payloads.

## Required headers

| Header | Purpose |
|--------|---------|
| `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>` | Runtime API authentication |
| `X-ArcFlow-Signature` | HMAC-SHA256 over the exact request body bytes |
| `Content-Type: application/json` | Outcome JSON |
| `X-Idempotency-Key` | Optional; prevents duplicate processing on retries |

## HMAC computation

ArcFlow signs the **raw JSON body bytes**, not a canonicalized re-serialization after parsing.

Algorithm:

1. Read the exact POST body as bytes.
2. Compute HMAC-SHA256 with `ARCFLOW_WEBHOOK_SECRET`.
3. Encode digest as lowercase hex.
4. Send header `X-ArcFlow-Signature: sha256=<hex>`.

Server verification lives in `runtime/arcflow-core/src/external/webhook.rs` via `verify_webhook_signature`. The header may be raw hex or the `sha256=` prefix form.

### Python (integrator signing)

```python
import hashlib
import hmac
import json

def sign_body(secret: str, body: bytes) -> str:
    digest = hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()
    return f"sha256={digest}"

outcome = {"binding_id": "payment_webhook", "status": "success", "fields": {"transaction_id": "tx_123"}}
body = json.dumps(outcome).encode()
signature = sign_body("your-webhook-secret", body)
```

Use the same helper inside `report_outcome` in `sdk-python/arcflow/external.py` when posting from Python integrators.

### TypeScript (integrator signing)

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

### Verifying inbound callbacks (custom gateway)

If you terminate TLS on a gateway before ArcFlow, verify at the gateway or rely on the server handler. Example verifier matching engine behavior:

```python
import hashlib
import hmac

def verify_webhook_signature(secret: str, body: bytes, signature_header: str) -> bool:
    expected = hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()
    provided = signature_header.strip().removeprefix("sha256=")
    return hmac.compare_digest(provided, expected)
```

## Constant-time comparison

The engine compares expected and provided digests with `subtle::ConstantTimeEq` to reduce timing side channels. Integrators should use `hmac.compare_digest` in Python or `timingSafeEqual` in Node when verifying signatures on their side.

Reject signatures whose hex length does not match the expected digest length before comparison.

## Server configuration

Set on `arcflow-server`:

```bash
ARCFLOW_WEBHOOK_SECRET=<long-random-string>
ARCFLOW_SERVER_API_KEY=<runtime-api-key>
```

If `ARCFLOW_WEBHOOK_SECRET` is unset, external callback routes return **503** with a clear error. Postgres must also be available.

## Secret rotation and dual-verify window

Production rotations should use a short overlap window:
