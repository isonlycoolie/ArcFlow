"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.openGraphPanel = openGraphPanel;
exports.isArcflowWorkflowDocument = isArcflowWorkflowDocument;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const layout_1 = require("./layout");
let activePanel;
function openGraphPanel(context, document) {
    const workflow = parseWorkflowDocument(document);
    if (!workflow) {
        return;
    }
    const layout = (0, layout_1.computeGraphLayout)(workflow);
    const dims = (0, layout_1.layoutDimensions)(layout);
    if (activePanel) {
        activePanel.reveal(vscode.ViewColumn.Beside);
        activePanel.webview.postMessage({ type: "update", layout, dims });
        return;
    }
    activePanel = vscode.window.createWebviewPanel("arcflowGraph", `ArcFlow: ${workflow.name}`, vscode.ViewColumn.Beside, {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [
            vscode.Uri.file(path.join(context.extensionPath, "media")),
        ],
    });
    activePanel.webview.html = buildGraphHtml(activePanel.webview, context.extensionPath, layout, dims);
    activePanel.onDidDispose(() => {
        activePanel = undefined;
    });
    activePanel.webview.onDidReceiveMessage((message) => {
        if (message.type === "ready") {
            activePanel?.webview.postMessage({ type: "update", layout, dims });
        }
    });
}
function parseWorkflowDocument(document) {
    try {
        const parsed = JSON.parse(document.getText());
        if (!parsed || typeof parsed !== "object" || !Array.isArray(parsed.steps)) {
            throw new Error("Invalid workflow: expected steps array.");
        }
        return parsed;
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        void vscode.window.showErrorMessage(`ArcFlow: cannot parse workflow JSON — ${message}`);
        return undefined;
    }
}
function buildGraphHtml(webview, extensionPath, layout, dims) {
    const cssUri = webview.asWebviewUri(vscode.Uri.file(path.join(extensionPath, "media", "graph.css")));
    const jsUri = webview.asWebviewUri(vscode.Uri.file(path.join(extensionPath, "media", "graph.js")));
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
<body>
  <header class="toolbar">
    <span id="workflow-name"></span>
    <span id="mode-badge" class="badge"></span>
    <span class="hint">Read-only preview</span>
  </header>
  <div id="warnings"></div>
  <div class="canvas-wrap">
    <svg id="graph-canvas" role="img" aria-label="Workflow graph"></svg>
  </div>
  <script nonce="${nonce}">
    window.__ARCFLOW_INITIAL__ = ${initialPayload};
  </script>
  <script nonce="${nonce}" src="${jsUri}"></script>
</body>
</html>`;
}
function getNonce() {
    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let nonce = "";
    for (let i = 0; i < 32; i++) {
        nonce += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return nonce;
}
function isArcflowWorkflowDocument(document) {
    return document.fileName.endsWith(".arcflow.json");
}
//# sourceMappingURL=webviewPanel.js.map