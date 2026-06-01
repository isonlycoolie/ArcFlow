# ArcFlow CLI

```bash
cargo install --path cli/arcflow-cli
arcflow init my-project
cd my-project
# Python: pip install arcflow && python workflows/example_workflow.py
arcflow trace <run-id>
```

```bash
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow
arcflow migrate up
```

Commands: `init`, `run`, `trace`, `validate`, `migrate up`, `schedule validate`. See [CLI overview](../../../cli/overview.md).
