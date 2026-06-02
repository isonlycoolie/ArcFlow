# RUNTIME-SERVER-API v1

**Status:** Implemented

| Method | Path | Auth |
|--------|------|------|
| GET | `/health` | No |
| GET | `/ready` | No |
| POST | `/v1/workflows/run` | `X-ArcFlow-Api-Key` |

Execution failures return HTTP 200 with error in RCS `RunResult` body.
