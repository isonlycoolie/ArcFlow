
# arcflow validate

Validate workflow definition files before commit or CI deploy. **Current status: stub (CLI validate command).** The command checks file readability only; it does **not** validate against [workflow schema](../contracts/rcs-schema.md) yet.

.

## Usage

```bash
arcflow validate WORKFLOW_FILE
```

Example:

```bash
arcflow validate workflows/support_router.json
```

## Current behavior

If file reads successfully and is non-empty:

```text
[ArcFlow] Workflow file is readable (syntax validation via Python SDK pending).
```

Exit code **0**.

Empty file:

```text
[ArcFlow] Workflow file is empty.
```

Exit code **4**.

Missing file:

```text
[ArcFlow] Cannot read workflows/missing.json:...
```

Exit code **1**.

## What CLI validate command will add

Target behavior (not shipped):

- Parse JSON/YAML workflow against workflow specification JSON Schema
- Report `WorkflowConfigurationError` paths with line hints
- `--format json` machine-readable diagnostics for CI
- Exit code 4 on schema violations

Do not rely on `arcflow validate` for production gates until CLI validate command closes.

## CI workaround today

Validate against normative schema with a JSON Schema tool:

```bash
# Example using npx (adjust to your CI runner)
npx ajv validate -s contracts/normative/rcs/v1.schema.json -d workflows/demo.json
```

Or Python:

```python
import json
import jsonschema

schema = json.load(open("contracts/normative/rcs/v1.schema.json"))
doc = json.load(open("workflows/demo.json"))
jsonschema.validate(doc, schema)
```

Integration tests with stub provider (`exec_config.test`) complement schema checks. See [guides/workflows/validation-and-testing.md](../guides/workflows/validation-and-testing.md).

## Related commands

`arcflow schedule validate MANIFEST` validates cron schedule manifests separately (implemented).

```bash
arcflow schedule validate schedules/nightly.yaml
```

## Related pages

- [maturity-and-known-gaps.md](../concepts/maturity-and-known-gaps.md) (CLI validate command)
- [cli/overview.md](overview.md)
- [guides/workflows/validation-and-testing.md](../guides/workflows/validation-and-testing.md)
