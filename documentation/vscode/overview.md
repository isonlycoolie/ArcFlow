
# VS Code extension overview

The in-repo VS Code extension (`extensions/vscode-arcflow/`) adds workflow authoring and debug tooling on top of the TypeScript SDK and `arcflow-core`. It is **not** marketplace GA; install from a local VSIX or workspace development host.

Version: `0.2.0-stable` (see `package.json`).

## Features

| Feature | Command / trigger | Purpose |
|---------|-------------------|---------|
| Graph visualization | `ArcFlow: Visualize Graph` | Render workflow graph DAG from workflow JSON |
| Trace timeline | `ArcFlow: View Trace Timeline` | Inspect metadata-only trace events over time |
| Local debug run | `ArcFlow: Start Debug Run` | Execute workflow from workspace |
| Breakpoints | `ArcFlow: Toggle Breakpoint` | Pause on steps in debug sessions |
| Server connect | `ArcFlow: Connect to Local Server` | Point extension at `arcflow-server` |

## Supported file types

| Language id | Extension | Content |
|-------------|-----------|---------|
| `arcflow-workflow` | `.arcflow.json` | Workflow specification definitions |
| `arcflow-trace` | `.arcflow.trace.json` | Exported execution traces |

Editor title bar buttons appear when filename matches `*.arcflow.json` or `*.arcflow.trace.json`.

## Installation (local)

From repository root:

```bash
cd extensions/vscode-arcflow
npm install
npm run compile
```

Launch Extension Development Host from VS Code (F5) or package VSIX:

```bash
npx vsce package
code --install-extension vscode-arcflow-0.2.0-stable.vsix
```

Requires VS Code **1.85+** and built `@arcflow/sdk` / native bindings for debug run features.

## Primary use cases

**Author graph workflows visually.** Open a graph-mode `.arcflow.json` and run Visualize Graph to validate node layout and edges before server deploy.

**Debug locally.** Start Debug Run invokes TS SDK against embedded core without Postgres (unless recovery features enabled).

**Read traces after failures.** Export trace JSON from server (`GET /v1/runs/{id}/trace`) or CLI (`arcflow trace --format json`), save as `.arcflow.trace.json`, open timeline view.

## What the extension does not replace

| Need | Use instead |
|------|-------------|
| Production HTTP integration | [HTTP API reference](../server/http-api-reference.md) |
| Operator site admin | Admin API or private dashboard |
| Browser chat widget | [Static product overview](../static-product/overview.md) |
| Server SSE streaming | Deferred server streaming |

## Tests

```bash
cd extensions/vscode-arcflow
npm test
```

Uses `@vscode/test-electron` per `package.json` scripts.

## Related pages

- [Graph view](graph-view.md)
- [Trace timeline](trace-timeline.md)
- [Trace command](../cli/trace.md)
