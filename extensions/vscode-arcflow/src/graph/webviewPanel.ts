import * as vscode from "vscode";
import * as path from "path";
import type { WorkflowDefinition } from "../types/rcs";
import { computeGraphLayout, layoutDimensions } from "./layout";

let activePanel: vscode.WebviewPanel | undefined;

export function openGraphPanel(
  context: vscode.ExtensionContext,
  document: vscode.TextDocument,
): void {
  const workflow = parseWorkflowDocument(document);
  if (!workflow) {
    return;
  }

  const layout = computeGraphLayout(workflow);
  const dims = layoutDimensions(layout);

  if (activePanel) {
    activePanel.reveal(vscode.ViewColumn.Beside);
    activePanel.webview.postMessage({ type: "update", layout, dims });
    return;
  }

  activePanel = vscode.window.createWebviewPanel(
    "arcflowGraph",
    `ArcFlow: ${workflow.name}`,
    vscode.ViewColumn.Beside,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [
        vscode.Uri.file(path.join(context.extensionPath, "media")),
      ],
    },
  );

  activePanel.webview.html = buildGraphHtml(
    activePanel.webview,
    context.extensionPath,
    layout,
    dims,
  );

  activePanel.onDidDispose(() => {
    activePanel = undefined;
  });

  activePanel.webview.onDidReceiveMessage((message: { type: string }) => {
    if (message.type === "ready") {
      activePanel?.webview.postMessage({ type: "update", layout, dims });
    }
  });
}

function parseWorkflowDocument(document: vscode.TextDocument): WorkflowDefinition | undefined {
  try {
    const parsed = JSON.parse(document.getText()) as WorkflowDefinition;
    if (!parsed || typeof parsed !== "object" || !Array.isArray(parsed.steps)) {
      throw new Error("Invalid workflow: expected steps array.");
    }
    return parsed;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    void vscode.window.showErrorMessage(`ArcFlow: cannot parse workflow JSON — ${message}`);
    return undefined;
  }
}

function buildGraphHtml(
  webview: vscode.Webview,
  extensionPath: string,
  layout: ReturnType<typeof computeGraphLayout>,
  dims: ReturnType<typeof layoutDimensions>,
): string {
  const cssUri = webview.asWebviewUri(
    vscode.Uri.file(path.join(extensionPath, "media", "graph.css")),
  );
  const jsUri = webview.asWebviewUri(
    vscode.Uri.file(path.join(extensionPath, "media", "graph.js")),
  );
  const nonce = getNonce();
  const initialPayload = JSON.stringify({ layout, dims });

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy"
    content="default-src 'none'; style-src ${webview.cspSource}; script-src 'nonce-${nonce}';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="stylesheet" href="${cssUri}">
  <title>ArcFlow Graph</title>
</head>
