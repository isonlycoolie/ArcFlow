**Audience:** `[developer]` `[operator]`

# arcflow trace

Print execution traces for a run id from in-process store, JSON file, or HTTP server.

## Usage

```bash
arcflow trace RUN_ID [options]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--format` | `human` | `human` or `json` |
| `--verbose` | off | Print raw events JSON after summary |
| `--tui` | off | Interactive terminal timeline |
| `--file PATH` | off | Load trace from JSON file |
| `--server URL` | off | Fetch `GET /v1/runs/{id}/trace` |

Global: `arcflow --no-color trace ...` for CI logs.

## Examples

In-process trace (same machine as SDK run):

```bash
arcflow trace 7c9e6679-7425-40de-944b-e07fc1f90ae7
```

JSON export:

```bash
arcflow trace 7c9e6679-7425-40de-944b-e07fc1f90ae7 --format json
```

Server-backed trace:

```bash
export ARCFLOW_SERVER_API_KEY=dev-secret
arcflow trace 7c9e6679-7425-40de-944b-e07fc1f90ae7 --server http://localhost:8080
```

Equivalent curl:

```bash
curl -s "http://localhost:8080/v1/runs/7c9e6679-7425-40de-944b-e07fc1f90ae7/trace" \
  -H "Authorization: Bearer dev-secret"
```

Import from file (export saved earlier):

```bash
arcflow trace any-id --file ./trace-export.json --format json
```

TUI viewer:

```bash
arcflow trace 7c9e6679-7425-40de-944b-e07fc1f90ae7 --tui
```

## Human output format

```text
run_id: 7c9e6679-7425-40de-944b-e07fc1f90ae7
workflow: research_pipeline
steps: 2
```

With `--verbose`, appends:

```text
events:
[
  { "kind": "WorkflowStarted", "run_id": "...", "workflow_name": "research_pipeline", "step_count": 2 },
  { "kind": "StepCompleted", ... },
  { "kind": "WorkflowCompleted", ... }
]
```

Events are SEC-1 metadata only (no prompt text).

## Where to find run IDs

| Source | Location |
|--------|----------|
| Server create response | `run_id` field from `POST /v1/runs` |
| Python SDK | `result.run_id` when exposed |
| Postgres | `arcflow_runs.run_id` |
| Docker logs | Server startup run logs |

## Exit codes

| Code | Meaning |
