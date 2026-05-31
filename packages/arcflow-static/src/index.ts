export interface ArcFlowClientOptions {
  baseUrl: string;
  apiKey: string;
}

export interface RunRequest {
  workflow: string;
  input: string;
  initialState?: Record<string, unknown>;
}

export interface RunResponse {
  run_id: string;
  status: string;
}

export class ArcFlowClient {
  constructor(private readonly opts: ArcFlowClientOptions) {}

  async run(req: RunRequest): Promise<RunResponse> {
    const res = await fetch(`${this.opts.baseUrl.replace(/\/$/, "")}/v1/runs`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${this.opts.apiKey}`,
      },
      body: JSON.stringify({
        workflow: req.workflow,
        input: req.input,
        initial_state: req.initialState,
      }),
    });
    if (!res.ok) {
      throw new Error(`ArcFlow run failed: ${res.status} ${await res.text()}`);
    }
    return (await res.json()) as RunResponse;
  }

  async pollTrace(runId: string, intervalMs = 500, maxAttempts = 60): Promise<unknown> {
    for (let i = 0; i < maxAttempts; i++) {
      const res = await fetch(
        `${this.opts.baseUrl.replace(/\/$/, "")}/v1/runs/${runId}/trace`,
        { headers: { Authorization: `Bearer ${this.opts.apiKey}` } },
      );
      if (res.ok) {
        const trace = await res.json();
        const status = (trace as { status?: string }).status;
        if (status === "Completed" || status === "Failed") {
          return trace;
        }
      }
      await new Promise((r) => setTimeout(r, intervalMs));
    }
    throw new Error(`Trace poll timeout for run ${runId}`);
  }
}

export interface ConversationTurn {
  role: "user" | "assistant";
  content: string;
}

export class StepForm {
  private turns: ConversationTurn[] = [];

  addTurn(role: ConversationTurn["role"], content: string): this {
    this.turns.push({ role, content });
    return this;
  }

  toInitialState(): Record<string, unknown> {
    return { conversation_turns: this.turns };
  }
}

export type { ExternalBinding, ExternalOutcomeReport } from "./types.js";
