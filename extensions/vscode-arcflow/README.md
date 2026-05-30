# ArcFlow for VS Code (Stable)

Local workflow graph, trace timeline, and step-through debugging. No cloud account required.

## Features

- Open `*.arcflow.json` → SVG graph webview (graph or linear layout)
- Layout positions saved to `.arcflow.layout.json` sidecar
- Open `*.arcflow.trace.json` → step timeline
- **Toggle Breakpoint** on step/node ids
- **Start Debug Run** via `POST /v1/debug/runs/start` (requires `ARCFLOW_DEBUG=true` on server)
- **Debug State** panel with masked step outputs at pause
- **Connect to Local Server** — pings `http://127.0.0.1:8080/health` (localhost only)

## Quick start

1. `npm install && npm run compile` in this folder
2. Start `arcflow-server` with `ARCFLOW_DEBUG=true`
3. Run **Extension Development Host** (F5)
4. Open `examples/react-preview.arcflow.json`

## Debug protocol

| Endpoint | Purpose |
|----------|---------|
| `POST /v1/debug/runs/start` | Start run with breakpoints |
| `GET /v1/debug/runs/{id}/state` | Paused state (values masked) |
| `POST /v1/debug/runs/{id}/continue` | Resume after breakpoint |

Debug routes bind only on localhost and are disabled unless `ARCFLOW_DEBUG=true`. Not for production deploy.

## Tests

```bash
npm test
```

## Marketplace

Publish manually when ready; stable code ships in-repo first.
