import * as vscode from "vscode";
import * as path from "path";
import type { ExecutionTrace } from "../types/rcs";

let activePanel: vscode.WebviewPanel | undefined;

/**
 * Trace timeline webview stub (Month 5 preview).
 * Renders step list and timestamps; step-through debugging arrives in Month 6.
 */
export function openTraceTimelinePanel(
  context: vscode.ExtensionContext,
  document: vscode.TextDocument,
): void {
  const trace = parseTraceDocument(document);
  if (!trace) {
    return;
  }

  const title = `ArcFlow Trace: ${trace.workflow_name || trace.run_id}`;

  if (activePanel) {
    activePanel.title = title;
    activePanel.reveal(vscode.ViewColumn.Beside);
    activePanel.webview.postMessage({ type: "update", trace });
    return;
  }

  activePanel = vscode.window.createWebviewPanel(
    "arcflowTrace",
    title,
    vscode.ViewColumn.Beside,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [
        vscode.Uri.file(path.join(context.extensionPath, "media")),
      ],
    },
  );

  activePanel.webview.html = buildTraceHtml(
    activePanel.webview,
    context.extensionPath,
    trace,
  );

  activePanel.onDidDispose(() => {
    activePanel = undefined;
  });
}

function parseTraceDocument(document: vscode.TextDocument): ExecutionTrace | undefined {
  try {
    const parsed = JSON.parse(document.getText()) as ExecutionTrace;
    if (!parsed || typeof parsed !== "object" || !parsed.run_id) {
      throw new Error("Invalid trace: expected run_id.");
    }
    if (!Array.isArray(parsed.steps)) {
      parsed.steps = [];
    }
    return parsed;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    void vscode.window.showErrorMessage(`ArcFlow: cannot parse trace JSON — ${message}`);
    return undefined;
  }
}

function buildTraceHtml(
  webview: vscode.Webview,
  extensionPath: string,
  trace: ExecutionTrace,
): string {
  const cssUri = webview.asWebviewUri(
    vscode.Uri.file(path.join(extensionPath, "media", "trace.css")),
  );
  const nonce = getNonce();
  const payload = JSON.stringify(trace);

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy"
    content="default-src 'none'; style-src ${webview.cspSource}; script-src 'nonce-${nonce}';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="stylesheet" href="${cssUri}">
  <title>ArcFlow Trace</title>
</head>
<body>
  <header class="toolbar">
    <h1>Trace Timeline</h1>
    <span class="stub-badge">Preview stub</span>
  </header>
