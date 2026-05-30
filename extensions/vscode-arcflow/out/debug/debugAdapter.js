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
exports.DebugAdapter = exports.BreakpointManager = void 0;
exports.openDebugStatePanel = openDebugStatePanel;
const vscode = __importStar(require("vscode"));
/** Manages step-id breakpoints for debug runs. */
class BreakpointManager {
    breakpoints = new Set();
    toggle(stepId) {
        if (this.breakpoints.has(stepId)) {
            this.breakpoints.delete(stepId);
            return false;
        }
        this.breakpoints.add(stepId);
        return true;
    }
    list() {
        return [...this.breakpoints];
    }
    clear() {
        this.breakpoints.clear();
    }
}
exports.BreakpointManager = BreakpointManager;
/** Thin client over arcflow-server debug endpoints. */
class DebugAdapter {
    client;
    constructor(client) {
        this.client = client;
    }
    async startRun(payload) {
        this.client.assertLocalhost();
        const response = await fetch(`${this.client.getBaseUrl()}/v1/debug/runs/start`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
            signal: AbortSignal.timeout(10000),
        });
        if (!response.ok) {
            return undefined;
        }
        const body = (await response.json());
        return body.run_id;
    }
    async fetchState(runId) {
        this.client.assertLocalhost();
        const response = await fetch(`${this.client.getBaseUrl()}/v1/debug/runs/${runId}/state`, { signal: AbortSignal.timeout(5000) });
        if (!response.ok) {
            return undefined;
        }
        const body = (await response.json());
        return body.state;
    }
    async continueRun(runId) {
        this.client.assertLocalhost();
        const response = await fetch(`${this.client.getBaseUrl()}/v1/debug/runs/${runId}/continue`, { method: "POST", signal: AbortSignal.timeout(5000) });
        return response.status === 204 || response.ok;
    }
}
exports.DebugAdapter = DebugAdapter;
async function openDebugStatePanel(context, state) {
    const panel = vscode.window.createWebviewPanel("arcflowDebugState", "ArcFlow Debug State", vscode.ViewColumn.Two, { enableScripts: false });
    const rows = state.masked_outputs
        .map((o) => `<tr><td>${o.step_id}</td><td>${o.agent_id}</td><td>${o.content_preview}</td><td>${o.status}</td></tr>`)
        .join("");
    panel.webview.html = `<!DOCTYPE html><html><body>
    <h2>Paused at step ${state.step_id}</h2>
    <p>Run: ${state.run_id} · index ${state.step_index}</p>
    <table border="1" cellpadding="4"><thead><tr><th>Step</th><th>Agent</th><th>Preview</th><th>Status</th></tr></thead>
    <tbody>${rows}</tbody></table>
  </body></html>`;
    context.subscriptions.push(panel);
}
//# sourceMappingURL=debugAdapter.js.map