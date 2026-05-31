# Track H: IDE and CLI

**Audience:** `[developer]`

Track H introduces developer tooling: VS Code workflow preview, local CLI execution, and comparison between CLI trace output and SDK results.

## Goal

Open a graph workflow in VS Code, view the graph, run a workflow locally via CLI, and compare CLI output to SDK output. First contact with ArcFlow developer tooling beyond raw scripts.

## Prerequisites

| Item | Required |
|------|----------|
| [Track A](track-a-first-workflow.md) | SDK workflow basics |
| Rust toolchain | `cargo run -p arcflow-cli` |
| VS Code | With ArcFlow extension from `extensions/vscode-arcflow/` |
| Graph sample | [`examples/graph/reflection_loop.py`](../../examples/graph/reflection_loop.py) or extension preview JSON |
| Track D | Helpful for graph semantics |

## Step 1: Open graph workflow in VS Code

Install or launch the ArcFlow extension from `extensions/vscode-arcflow/`. Open a workflow definition:

| Path | Purpose |
|------|---------|
| [`extensions/vscode-arcflow/examples/react-preview.arcflow.json`](../../extensions/vscode-arcflow/examples/react-preview.arcflow.json) | Extension graph preview sample |
| [`examples/graph/reflection_loop.py`](../../examples/graph/reflection_loop.py) | Python graph source to compare |

Use the extension graph panel to inspect nodes and edges. Confirm entry node and conditional edges match the Python DSL in reflection loop.

## Step 2: Run workflow via SDK (baseline)

```bash
python examples/graph/reflection_loop.py
```

Record `run_id` and `step_count` from stdout.

Optional assertions from Track A pattern:

```python
assert result.status == "completed"
assert result.run_id
```

## Step 3: Run via CLI

From repository root, initialize a minimal project if needed:

```bash
cargo run -p arcflow-cli -- init my-track-h --language python
```

Run a workflow file through CLI (exact subcommand depends on your checkout; common pattern):

```bash
cargo run -p arcflow-cli -- run examples/graph/reflection_loop.py
```

If your CLI version expects a manifest path, point at the generated project or script per `extensions/vscode-arcflow/README.md` and CLI help:

```bash
cargo run -p arcflow-cli -- --help
cargo run -p arcflow-cli -- run --help
```

Pass criteria: CLI exits zero and prints run summary with run id.

## Step 4: Compare CLI trace to SDK trace

Export trace for the run id from either path:

```bash
cargo run -p arcflow-cli -- trace YOUR_RUN_ID --format json --verbose
```

Compare event kinds to SDK:

```python
kinds = {e.get("event_kind") for e in result.trace_events}
print(sorted(kinds))
```

Expect matching lifecycle and graph node kinds for the same logical workflow. Token counts and durations may differ slightly between invocations but structure should align.

## Step 5: VS Code run integration (optional)

If the extension provides run commands, trigger run from the editor and open trace view. Compare displayed graph highlight order to `GraphNodeStarted` sequence in exported JSON.

## Verification checklist

| Check | Expected |
|-------|----------|
| Graph preview | Nodes and edges visible |
