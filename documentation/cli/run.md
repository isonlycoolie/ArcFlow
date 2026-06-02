
# arcflow run

Execute a workflow file from the terminal. The CLI entry point exists for a uniform developer experience; current implementation directs you to the Python SDK for actual execution.

## Usage

```bash
arcflow run WORKFLOW_FILE [--input TEXT]
```

| Flag | Description |
|------|-------------|
| `WORKFLOW_FILE` | Path to `.py` or workflow script |
| `--input` | Optional run input string (workflow must read it) |

## Current behavior

```bash
arcflow run workflows/example_workflow.py --input "hello"
```

Prints guidance to invoke the Python SDK:

```text
[ArcFlow] arcflow run for workflows/example_workflow.py requires the Python SDK (arcflow package).
 Use: python -c "import runpy; runpy.run_path('workflows/example_workflow.py')"
[ArcFlow] --input is accepted; wire your workflow entrypoint to call workflow.run().
```

Exit code **2** (execution not completed via CLI native path).

## Recommended local execution

Python:

```python
from arcflow import Agent, Workflow

wf = Workflow("demo")
wf.step(Agent(name="writer", role="author", instructions="Summarize."))
result = wf.run("hello")
print(result.output)
```

Or run the workflow file directly:

```bash
python workflows/example_workflow.py
```

TypeScript: use `@arcflow/sdk` with `node` or `tsx` per [TypeScript quickstart](../getting-started/quickstart-typescript.md).

## Remote runtime (--runtime planned)

 spec includes `--runtime http://host:8080` for server-backed runs. When wired, pattern will be:

```bash
export ARCFLOW_SERVER_API_KEY=dev-secret
arcflow run workflows/demo.py --runtime http://localhost:8080 --input "hello"
```

Until native remote run ships, use curl `POST /v1/runs` or Python `Workflow(..., runtime="http://localhost:8080")`.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success (when native execution implemented) |
| 2 | Execution failure or SDK delegation message |
| 3 | Infrastructure / parse error |

## Related pages

- [Init command](init.md)
- [Python quickstart](../getting-started/quickstart-python.md)
- [HTTP API reference](../server/http-api-reference.md)
