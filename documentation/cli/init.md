**Audience:** `[developer]`

# arcflow init

Scaffolds a new ArcFlow project directory with workflow folders, example stub file, and minimal config. Works offline (no network calls).

## Usage

```bash
arcflow init [OUTPUT_DIR] [options]
```

| Argument / flag | Default | Description |
|-----------------|---------|-------------|
| `OUTPUT_DIR` | `my-arcflow-project` | Target directory |
| `--lang` | `python` | `python` or `typescript` (file extension) |
| `--force` | off | Overwrite non-empty directory |

## Examples

Python project:

```bash
arcflow init my-bot --lang python
```

TypeScript project:

```bash
arcflow init my-bot --lang typescript
```

Force into existing folder:

```bash
arcflow init ./existing --force
```

## Generated structure

```text
my-arcflow-project/
  arcflow.config.yaml
  workflows/
    example_workflow.py   # or .ts
  agents/
  tools/
```

**arcflow.config.yaml** (initial content):

```yaml
runtime_mode: embedded
```

**workflows/example_workflow.{py|ts}** contains a one-line comment pointing to `arcflow run`.

## Expected output

```text
[ArcFlow] Created project at my-arcflow-project. Next: cd my-arcflow-project && arcflow run workflows/example_workflow.py
```

## Exit codes

| Code | Cause |
|------|-------|
| 0 | Success |
| 1 | Directory not empty without `--force` |
| 3 | Filesystem error creating dirs or files |

## Next steps after init

1. Replace example workflow with real `Workflow` + `Agent` definitions (Python or TypeScript SDK).
2. Copy `.env.example` patterns from SDK README for provider keys.
3. Run via SDK or `arcflow run` (see [run.md](run.md)).

`init` does not install Python packages or npm modules. Install SDK separately per [getting-started/install-and-build.md](../getting-started/install-and-build.md).

## Related pages

- [cli/overview.md](overview.md)
- [cli/run.md](run.md)

**Source:** capabilities reference §18; `cli/arcflow-cli/src/commands/init.rs`; ADR-018, Sprint 8 Phase 2.
