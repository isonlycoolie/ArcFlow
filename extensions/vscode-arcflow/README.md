# ArcFlow for VS Code (Preview)

Month 5 preview: read-only workflow graph and trace timeline for local development. No cloud account required.

## Features

- Open `*.arcflow.json` → SVG graph webview (graph or linear layout)
- Open `*.arcflow.trace.json` → step timeline stub
- **ArcFlow: Connect to Local Server** — pings `http://127.0.0.1:8080/health` (localhost only)

## Quick start

1. `npm install && npm run compile` in this folder
2. Run **Extension Development Host** (F5) from `extensions/vscode-arcflow`
3. Open `examples/react-preview.arcflow.json` or `examples/react-preview.arcflow.trace.json`

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `arcflow.serverUrl` | `http://127.0.0.1:8080` | Local server URL (127.0.0.1 / localhost only) |
| `arcflow.autoOpenGraph` | `true` | Open graph when a workflow file opens |
| `arcflow.autoOpenTrace` | `true` | Open timeline when a trace file opens |

## Stable (Month 6)

Breakpoints on node/step ids, state inspection, debug console, and debug adapter wiring to `/v1/debug/runs/*` endpoints.
