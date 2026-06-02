# CHAOS-TEST-SPEC v1

Deterministic failure injection for fault tolerance testing. No live provider APIs.

## Rules

- Use `wiremock`, `MockToolProvider`, or in-process counters only.
- No `std::thread::sleep` for synchronization; use zero-delay backoff in tests.
- Each scenario maps to one Rust integration test under `runtime/arcflow-core/tests/chaos_*.rs`.

## Scenarios

| ID | Scenario | Pass criteria |
|----|----------|---------------|
| C1 | Provider rate limit then success | `execute_with_retry` succeeds; attempts == 2 |
| C2 | Provider rate limit exhaust | `RetryExhausted`; attempts == max |
| C3 | Non-retryable API error | Single attempt; no backoff delay |
| C4 | Constant backoff timing | Delays match config within tolerance |
| C5 | Workflow timeout | `TimeoutEnforced` trace or timeout error |
| C6 | Step timeout | Step aborts before workflow budget |
| C7 | Tool execution failure | Step fails; retry if configured |
| C8 | Recovery resume | Completed steps not re-executed (Python + storage test) |
| C9 | Postgres unavailable at ready | `/ready` returns 503 `postgres_unavailable` |
| C10 | Migrations pending at ready | `/ready` returns 503 `migrations_pending` |

## Verification

```bash
cargo test -p arcflow-core chaos_
cd sdk-python && pytest tests/integration/test_fault_tolerance.py -q
```
