
# ArcFlow CLI overview

The `arcflow` binary (`cli/arcflow-cli/`) is a Rust command-line tool with no Python or Node runtime dependency. Startup time targets under 200ms for operator scripts and local developer workflows.

## Installation

From repository root after Rust toolchain install:

```bash
cargo build -p arcflow-cli --release
# binary: target/release/arcflow.exe (Windows) or target/release/arcflow
```

Run without install:

```bash
cargo run -p arcflow-cli -- --help
```

## Command index

| Command | Audience | Purpose |
|---------|----------|---------|
| `arcflow init` | developer | Scaffold workflow project |
| `arcflow run` | developer | Execute workflow file locally |
| `arcflow trace` | developer, operator | Inspect execution trace |
| `arcflow validate` | developer | **Stub (CLI validate command)** readability check only |
| `arcflow schedule validate` | developer | Validate cron schedule manifest |
| `arcflow migrate up` | operator, platform | Apply Postgres migrations |

Global flag: `--no-color` disables ANSI colors (CI friendly).

## Primary use cases

**Developers** scaffold projects, run workflows against embedded core (via Python SDK invocation path for `run`), and inspect traces after local or server runs.

**Operators** run `migrate up` in deploy pipelines before rolling `arcflow-server`, then verify `/ready`.

## Environment variables

| Variable | Commands |
|----------|----------|
| `ARCFLOW_POSTGRESQL_URL` | `migrate up` |
| `ARCFLOW_SERVER_API_KEY` | `trace --server URL` |

Full env reference: [Environment variables reference](../deployment/environment-variables-reference.md).

## Exit codes (summary)

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User error (file not found, validation) |
| 2 | Execution / infrastructure failure |
| 3 | CLI parse or infrastructure setup failure |

Per-command detail in linked pages.

## Known limitations

| Item | Status |
|------|--------|
| `arcflow validate` full JSON Schema | **Stub CLI validate command** |
| `arcflow run` native execution | Delegates to Python SDK message today |
| Graph/RAG via CLI run | Use SDK or server API |

## Related pages

| Page | Topic |
|------|-------|
| [init.md](init.md) | Project scaffold |
| [run.md](run.md) | Local execution |
| [trace.md](trace.md) | Trace inspection |
| [migrate.md](migrate.md) | Database migrations |
| [validate.md](validate.md) | Stub validate + CI workaround |
