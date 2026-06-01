# `arcflow trace` CLI reference

Inspect in-process workflow traces without Python.

## Usage

```bash
arcflow trace <run-id> [--format human|json] [--verbose] [--no-color]
```

Global flags apply to all subcommands.

| Flag | Description |
|------|-------------|
| `--format human` | Summary lines (default) |
| `--format json` | Full `ExecutionTrace` JSON |
| `--verbose` | After summary, print raw trace events |
| `--no-color` | Disable ANSI color (sets `NO_COLOR=1`) |

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Trace found and printed |
| `1` | Trace not found, or serialization failure |
| `2` | Trace store lock failed (poisoned mutex) |
| `3` | Invalid CLI arguments |

## Examples

```bash
# Human summary
arcflow trace 550e8400-e29b-41d4-a716-446655440000

# JSON for piping to jq
arcflow trace 550e8400-e29b-41d4-a716-446655440000 --format json

# Summary plus event list
arcflow trace 550e8400-e29b-41d4-a716-446655440000 --verbose
```

## Limitations (Sprint 5)

- Reads the **same process** store as the SDK. Traces from another process or host are not visible.
- Distributed tracing arrives in Sprint 8.
