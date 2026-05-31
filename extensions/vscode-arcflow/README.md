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

### Manual UI test (recommended)

Use **F5**, not `npm test`. Automated tests open a bare Extension Host with no sample folder.

1. Open **`extensions/vscode-arcflow`** as the VS Code workspace (File → Open Folder),  
   **or** open `arcflow-extension.code-workspace` (includes the `examples/` folder in the sidebar).
2. In a terminal, start the server (use a free port if 8080 is busy):

   ```powershell
   $env:ARCFLOW_DEBUG='true'
   $env:ARCFLOW_SERVER_API_KEY='dev-secret'
   cargo run -p arcflow-server
   ```

3. In VS Code: **Run and Debug** → **Run ArcFlow Extension** → F5.
4. In the new **[Extension Development Host]** window:
   - Open **Explorer** → **examples/** (or the **Sample workflows** root if you used the `.code-workspace` file)
   - Click **react-preview.arcflow.json** → graph webview opens beside the editor
5. Command Palette: **ArcFlow: Connect to Local Server**, then **Toggle Breakpoint** / **Start Debug Run**.

If you only see a title like `ARCFLOW-EXT-TEST-…` with no files, you launched via `npm test` or opened the repo root instead of the extension folder. Close that window and use step 1 + F5 above.

### Automated tests

```bash
npm install && npm test
```

Runs breakpoint unit tests inside a minimal Extension Host (no sample workspace).

### Legacy steps

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
