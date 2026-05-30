import * as vscode from "vscode";
import { ServerClient } from "../client/serverClient";

export interface DebugStateView {
  run_id: string;
  step_id: string;
  step_index: number;
  committed_step_ids: string[];
  masked_outputs: Array<{
    step_id: string;
    agent_id: string;
    content_preview: string;
    status: string;
  }>;
}

/** Manages step-id breakpoints for debug runs. */
export class BreakpointManager {
  private readonly breakpoints = new Set<string>();

  toggle(stepId: string): boolean {
    if (this.breakpoints.has(stepId)) {
      this.breakpoints.delete(stepId);
      return false;
    }
    this.breakpoints.add(stepId);
    return true;
  }

  list(): string[] {
    return [...this.breakpoints];
  }

  clear(): void {
    this.breakpoints.clear();
  }
}

/** Thin client over arcflow-server debug endpoints. */
export class DebugAdapter {
  constructor(private readonly client: ServerClient) {}

  async startRun(payload: {
    workflow: unknown;
    agents: unknown[];
    input: string;
    breakpoints: string[];
  }): Promise<string | undefined> {
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
    const body = (await response.json()) as { run_id?: string };
    return body.run_id;
  }

  async fetchState(runId: string): Promise<DebugStateView | undefined> {
    this.client.assertLocalhost();
    const response = await fetch(
      `${this.client.getBaseUrl()}/v1/debug/runs/${runId}/state`,
      { signal: AbortSignal.timeout(5000) },
    );
    if (!response.ok) {
      return undefined;
    }
    const body = (await response.json()) as {
      paused?: boolean;
      state?: DebugStateView;
    };
    return body.state;
  }

  async continueRun(runId: string): Promise<boolean> {
    this.client.assertLocalhost();
    const response = await fetch(
      `${this.client.getBaseUrl()}/v1/debug/runs/${runId}/continue`,
      { method: "POST", signal: AbortSignal.timeout(5000) },
    );
    return response.status === 204 || response.ok;
  }
}

export async function openDebugStatePanel(
  context: vscode.ExtensionContext,
  state: DebugStateView,
): Promise<void> {
  const panel = vscode.window.createWebviewPanel(
    "arcflowDebugState",
    "ArcFlow Debug State",
    vscode.ViewColumn.Two,
    { enableScripts: false },
  );
  const rows = state.masked_outputs
    .map(
      (o) =>
        `<tr><td>${o.step_id}</td><td>${o.agent_id}</td><td>${o.content_preview}</td><td>${o.status}</td></tr>`,
    )
    .join("");
  panel.webview.html = `<!DOCTYPE html><html><body>
    <h2>Paused at step ${state.step_id}</h2>
    <p>Run: ${state.run_id} · index ${state.step_index}</p>
    <table border="1" cellpadding="4"><thead><tr><th>Step</th><th>Agent</th><th>Preview</th><th>Status</th></tr></thead>
    <tbody>${rows}</tbody></table>
  </body></html>`;
  context.subscriptions.push(panel);
}
