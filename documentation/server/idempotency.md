
# Idempotency on run creation

`POST /v1/runs` accepts an optional `Idempotency-Key` header. When present, the server deduplicates create requests so network retries do not spawn duplicate workflow executions.

Relay forwards the same header upstream when browsers or BFFs set it on site-scoped create calls.

## Behavior

1. Client sends `POST /v1/runs` with `Idempotency-Key: <unique-string>` and a JSON body.
2. Server checks `arcflow_runs.idempotency_key` (UNIQUE column).
3. If the key already exists, the server returns **200/201-style** create response for the **original** run (`run_id`, `trace_id`, `status`) without starting a second execution.
4. If the key is new, the server inserts the run with that key and executes normally.

Implementation: `server/arcflow-server/src/handlers/runs.rs`, `server/arcflow-server/src/store/runs.rs`.

## Example: first request

```bash
curl -s -X POST http://localhost:8080/v1/runs \
 -H "Content-Type: application/json" \
 -H "Authorization: Bearer dev-secret" \
 -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000" \
 -d @run-payload.json
```

Response:

```json
{
 "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
 "trace_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
 "status": "Running"
}
```

## Example: retry with same key

Repeat the identical curl (same key, same body). The server returns the same `run_id` and does not create a second run.

```bash
curl -s -X POST http://localhost:8080/v1/runs \
 -H "Content-Type: application/json" \
 -H "Authorization: Bearer dev-secret" \
 -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000" \
 -d @run-payload.json
```

## Generating good keys

| Practice | Reason |
|----------|--------|
| Use UUID v4 or ULID | Globally unique per logical operation |
| One key per user action | Same checkout/submit gets one run |
| Include tenant id in key prefix | Avoid cross-tenant collision in shared logs |
| Do not reuse keys across different payloads | Idempotency is keyed only on header, not body hash today |

If two different payloads share a key, the second request still returns the first run. Keys must be unique per intended operation.

## Storage and retention

Idempotency keys persist in `arcflow_runs.idempotency_key` with a UNIQUE constraint. There is **no automatic TTL** in the current schema. Keys remain until the run row is deleted by operational cleanup. Plan retention policies accordingly for long-lived databases.

## External callbacks

External callback ingress supports `X-Idempotency-Key` on `POST /v1/runs/{run_id}/external/{binding_id}` with an in-memory dedup set for duplicate webhook delivery. That path is separate from run-create idempotency.

## Parity

| Surface | Idempotency-Key on create |
|---------|:-------------------------:|
| arcflow-server | Yes |
| arcflow-relay | Proxied |
| Python/TS SDK in-process | No (single process) |
| Static SDK | Optional via client if exposed |

## Related pages

- [http-api-reference.md](http-api-reference.md) for create-run contract
- [run-state-machine.md](run-state-machine.md) for polling after create
