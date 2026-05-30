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
exports.openTraceTimelinePanel = openTraceTimelinePanel;
exports.isArcflowTraceDocument = isArcflowTraceDocument;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
let activePanel;
/**
 * Trace timeline webview stub (Month 5 preview).
 * Renders step list and timestamps; step-through debugging arrives in Month 6.
 */
function openTraceTimelinePanel(context, document) {
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
    activePanel = vscode.window.createWebviewPanel("arcflowTrace", title, vscode.ViewColumn.Beside, {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [
            vscode.Uri.file(path.join(context.extensionPath, "media")),
        ],
    });
    activePanel.webview.html = buildTraceHtml(activePanel.webview, context.extensionPath, trace);
    activePanel.onDidDispose(() => {
        activePanel = undefined;
    });
}
function parseTraceDocument(document) {
    try {
        const parsed = JSON.parse(document.getText());
        if (!parsed || typeof parsed !== "object" || !parsed.run_id) {
            throw new Error("Invalid trace: expected run_id.");
        }
        if (!Array.isArray(parsed.steps)) {
            parsed.steps = [];
        }
        return parsed;
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        void vscode.window.showErrorMessage(`ArcFlow: cannot parse trace JSON — ${message}`);
        return undefined;
    }
}
function buildTraceHtml(webview, extensionPath, trace) {
    const cssUri = webview.asWebviewUri(vscode.Uri.file(path.join(extensionPath, "media", "trace.css")));
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
  <section id="meta"></section>
  <ol id="timeline" class="timeline"></ol>
  <p class="footer-note">Step-through debugging and breakpoints ship in stable (Month 6).</p>
  <script nonce="${nonce}">
    (function () {
      const trace = ${payload};
      const meta = document.getElementById("meta");
      const timeline = document.getElementById("timeline");

      function fmtTs(value) {
        if (!value) return "—";
        try { return new Date(value).toLocaleString(); } catch { return value; }
      }

      function render(t) {
        meta.innerHTML = [
          "<dl>",
          "<dt>Run ID</dt><dd>" + t.run_id + "</dd>",
          "<dt>Workflow</dt><dd>" + (t.workflow_name || "unknown") + "</dd>",
          "<dt>Status</dt><dd>" + (t.status || "unknown") + "</dd>",
          "<dt>Started</dt><dd>" + fmtTs(t.started_at) + "</dd>",
          "<dt>Completed</dt><dd>" + fmtTs(t.completed_at) + "</dd>",
          "<dt>Duration</dt><dd>" + (t.duration_ms != null ? t.duration_ms + " ms" : "—") + "</dd>",
          "</dl>"
        ].join("");

        timeline.innerHTML = "";
        if (!t.steps || t.steps.length === 0) {
          timeline.innerHTML = "<li class=\\"empty\\">No steps recorded.</li>";
          return;
        }
        t.steps.forEach(function (step) {
          const li = document.createElement("li");
          li.className = "step step-" + (step.status || "unknown").toLowerCase();
          li.innerHTML =
            "<div class=\\"step-head\\">" +
            "<strong>#" + step.step_index + " " + (step.agent_name || step.step_id) + "</strong>" +
            "<span class=\\"status\\">" + (step.status || "?") + "</span>" +
            "</div>" +
            "<div class=\\"step-meta\\">" +
            (step.agent_role ? "<span>" + step.agent_role + "</span>" : "") +
            "<span>" + fmtTs(step.started_at) + "</span>" +
            (step.duration_ms != null ? "<span>" + step.duration_ms + " ms</span>" : "") +
            "</div>";
          timeline.appendChild(li);
        });
      }

      render(trace);
      window.addEventListener("message", function (event) {
        if (event.data && event.data.type === "update") {
          render(event.data.trace);
        }
      });
    })();
  </script>
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
function isArcflowTraceDocument(document) {
    return document.fileName.endsWith(".arcflow.trace.json");
}
//# sourceMappingURL=traceTimelinePanel.js.map