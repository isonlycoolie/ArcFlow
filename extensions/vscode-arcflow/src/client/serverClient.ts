import * as vscode from "vscode";

const DEFAULT_URL = "http://127.0.0.1:8080";

/** Local-only HTTP client stub for arcflow-server (Month 5 preview). */
export class ServerClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl ?? DEFAULT_URL;
  }

  static fromConfig(): ServerClient {
    const config = vscode.workspace.getConfiguration("arcflow");
    const url = config.get<string>("serverUrl", DEFAULT_URL);
    return new ServerClient(url);
  }

  getBaseUrl(): string {
    return this.baseUrl;
  }

  /** Validates localhost binding before any request. */
  assertLocalhost(): void {
    let parsed: URL;
    try {
      parsed = new URL(this.baseUrl);
    } catch {
      throw new Error(`Invalid server URL: ${this.baseUrl}`);
    }
    const host = parsed.hostname;
    if (host !== "127.0.0.1" && host !== "localhost" && host !== "::1") {
      throw new Error(
        "ArcFlow debug endpoints must use localhost (127.0.0.1). Remote URLs are not permitted.",
      );
    }
  }

  /** Ping health endpoint; returns false when server is unreachable. */
  async ping(): Promise<boolean> {
    this.assertLocalhost();
    try {
      const response = await fetch(`${this.baseUrl}/health`, {
        method: "GET",
        signal: AbortSignal.timeout(3000),
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /** Fetch execution trace for a run id (stub — wired in stable release). */
  async fetchRunTrace(runId: string): Promise<unknown | undefined> {
    this.assertLocalhost();
    try {
      const response = await fetch(`${this.baseUrl}/v1/runs/${runId}/trace`, {
        method: "GET",
        signal: AbortSignal.timeout(5000),
      });
      if (!response.ok) {
        return undefined;
      }
      return response.json();
    } catch {
      return undefined;
    }
  }
}

export async function connectToLocalServer(): Promise<void> {
  const client = ServerClient.fromConfig();
  const ok = await client.ping();
  if (ok) {
    void vscode.window.showInformationMessage(
      `ArcFlow: connected to ${client.getBaseUrl()}`,
    );
  } else {
    void vscode.window.showWarningMessage(
      `ArcFlow: could not reach server at ${client.getBaseUrl()}. Start arcflow-server locally.`,
    );
  }
}
