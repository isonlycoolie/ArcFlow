# Track H: IDE and CLI


Track H introduces developer tooling: VS Code workflow preview, local CLI execution, and comparison between CLI trace output and SDK results.

## Goal

Open a graph workflow in VS Code, view the graph, run a workflow locally via CLI, and compare CLI output to SDK output. First contact with ArcFlow developer tooling beyond raw scripts.

## Prerequisites

| Item | Required |
|------|----------|
| [Track A](track-a-first-workflow.md) | SDK workflow basics |
| Rust toolchain | `cargo run -p arcflow-cli` |
| VS Code | With ArcFlow extension from `extensions/vscode-arcflow/` |
| Graph sample | [Graph routing walkthrough](../examples/graph-routing.md) or extension preview JSON |
| Track D | Helpful for graph semantics |

## Step 1: Open graph workflow in VS Code

Install or launch the ArcFlow extension from `extensions/vscode-arcflow/`. Open a workflow definition:

| Path | Purpose |
|------|---------|
| [`extensions/vscode-arcflow/examples/react-preview.arcflow.json`](../vscode/overview.md) | Extension graph preview sample |
| [Graph routing walkthrough](../examples/graph-routing.md) | Python graph source to compare |

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
| SDK run | `completed` status |
| CLI run | Zero exit, run id emitted |
| Trace kinds | SDK and CLI agree on major kinds |
| SEC-1 | Trace JSON has no prompt bodies |

## Expected output

SDK:

```
run_id=<uuid> steps=<n>
```

CLI trace export: JSON array of events with `kind` or `event_kind` fields per bridge version.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Extension graph empty | Invalid JSON or unsupported shape | Open `react-preview.arcflow.json` sample |
| CLI run not found | Wrong path or subcommand | Read `arcflow-cli --help` |
| Trace mismatch | Different run ids compared | Export trace for same uuid |
| `TraceNotFoundError` | In-process SDK run without server persist | Use server runtime or same process trace |

## What you learned

Track H connects authoring (IDE graph), execution (CLI), and observability (trace export). Teams use this loop for local debugging before promoting workflows to server registry or static publish.

## Next steps

| Topic | Link |
|-------|------|
| Server promotion | [Track B](track-b-server-api.md) |
| Certification Level 1 | [level-1-workflow-developer](../certification/level-1-workflow-developer.md) |
| Extension docs | `extensions/vscode-arcflow/README.md` |
