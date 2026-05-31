import {
  StaticConfigurationError,
  StaticExecutionError,
  WorkflowInterruptedError,
} from "./errors.js";
import type { ExternalOutcomeReport } from "./external.js";
import { parseCreateRun, parseRunStatus, type RunResult } from "./result.js";
import type { Workflow } from "./workflow.js";

export type ClientMode = "direct" | "relay" | "bff";

export interface ArcFlowClientOptions {
  baseUrl: string;
  apiKey: string;
  mode?: ClientMode;
  /** Site id when baseUrl is relay host root (optional if baseUrl includes /v1/sites/{id}). */
  siteId?: string;
  /** Prefer X-ArcFlow-Api-Key; relay mode defaults to Bearer. */
  useArcFlowHeader?: boolean;
}

const API_KEY_HEADER = "X-ArcFlow-Api-Key";

export class ArcFlowClient {
  private readonly baseUrl: string;
  private readonly apiKey: string;
  private readonly useArcFlowHeader: boolean;
  private readonly siteId: string | undefined;
  readonly mode: ClientMode;

  constructor(opts: ArcFlowClientOptions) {
    const base = opts.baseUrl.trim().replace(/\/$/, "");
    if (!base) {
      throw new StaticConfigurationError("[ArcFlow] ArcFlowClient baseUrl must be non-empty.");
    }
    if (!opts.apiKey.trim()) {
      throw new StaticConfigurationError("[ArcFlow] ArcFlowClient apiKey must be non-empty.");
    }
    this.baseUrl = base;
    this.apiKey = opts.apiKey.trim();
    const mode = opts.mode ?? "relay";
    this.mode = mode === "bff" ? "relay" : mode;
    this.siteId = opts.siteId ?? parseSiteIdFromBase(base);
    this.useArcFlowHeader =
      opts.useArcFlowHeader ?? (this.mode === "direct");
  }

  private headers(extra?: Record<string, string>): Record<string, string> {
    const h: Record<string, string> = {
      "Content-Type": "application/json",
      ...extra,
    };
    if (this.useArcFlowHeader) {
      h[API_KEY_HEADER] = this.apiKey;
    } else {
      h.Authorization = `Bearer ${this.apiKey}`;
    }
    return h;
  }

  private runsBasePath(): string {
    if (this.mode === "relay" && this.siteId) {
      return `/v1/sites/${encodeURIComponent(this.siteId)}/runs`;
    }
    return "/v1/runs";
  }

  async runWorkflow(
    workflow: Workflow,
    input: string,
    options: { initialState?: Record<string, unknown> } = {},
  ): Promise<RunResult> {
    const payload = workflow.buildRunPayload(input, options.initialState);
    const created = await this.request<Record<string, unknown>>("POST", this.runsBasePath(), payload);
    const { runId, status } = parseCreateRun(created);
    if (status.toLowerCase() === "completed" || status.toLowerCase() === "failed") {
      return this.getRun(runId);
    }
    if (status.toLowerCase() === "interrupted") {
      const detail = await this.getRun(runId);
      if (detail.interrupt) {
        throw new WorkflowInterruptedError(
          `[ArcFlow] Run '${runId}' paused for approval.`,
          runId,
          detail.interrupt.approvalKey,
          detail.interrupt.expiresAt,
        );
      }
    }
    return this.pollUntilComplete(runId);
  }

  async runPublished(
    name: string,
    version: string,
    input: string,
    options: { initialState?: Record<string, unknown> } = {},
  ): Promise<RunResult> {
    const payload: Record<string, unknown> = {
      workflow_ref: { name, version },
      input,
    };
    if (options.initialState) {
      payload.exec_config = { initial_state: options.initialState };
    }
    const created = await this.request<Record<string, unknown>>("POST", this.runsBasePath(), payload);
    const { runId, status } = parseCreateRun(created);
    if (status.toLowerCase() === "completed" || status.toLowerCase() === "failed") {
      return this.getRun(runId);
    }
    return this.pollUntilComplete(runId);
  }

  async getRun(runId: string): Promise<RunResult> {
    const path =
      this.mode === "relay" && this.siteId
        ? `/v1/sites/${encodeURIComponent(this.siteId)}/runs/${encodeURIComponent(runId)}`
        : `/v1/runs/${runId}`;
    const body = await this.request<Record<string, unknown>>("GET", path);
    const parsed = parseRunStatus(body);
    if (parsed.status.toLowerCase() === "failed" && parsed.error) {
      throw new StaticExecutionError(parsed.error.message, runId, parsed.error.stepId);
    }
    return parsed;
  }

  async pollUntilComplete(runId: string, intervalMs = 500, maxAttempts = 60): Promise<RunResult> {
    for (let i = 0; i < maxAttempts; i++) {
      const detail = await this.getRun(runId);
      const status = detail.status.toLowerCase();
      if (status === "completed") return detail;
      if (status === "failed") {
        throw new StaticExecutionError(detail.error?.message ?? "run failed", runId, detail.error?.stepId);
      }
      if (status === "interrupted" && detail.interrupt) {
        throw new WorkflowInterruptedError(
          `[ArcFlow] Run '${runId}' paused for approval.`,
          runId,
          detail.interrupt.approvalKey,
          detail.interrupt.expiresAt,
        );
      }
      await sleep(intervalMs);
    }
    throw new StaticExecutionError(`[ArcFlow] Poll timeout for run '${runId}'.`, runId);
  }

  async publishWorkflow(workflow: Workflow, version: string, publishedBy?: string): Promise<Record<string, unknown>> {
    if (!workflow.runtimeUrl) {
      throw new StaticConfigurationError("[ArcFlow] Workflow requires runtime URL for publish.");
    }
    const payload = workflow.buildPublishPayload(publishedBy);
    return this.request(
      "PUT",
      `/v1/workflows/${encodeURIComponent(workflow.name)}/versions/${encodeURIComponent(version)}`,
      payload,
    );
  }

  async resolveWorkflow(name: string, version: string): Promise<Record<string, unknown>> {
    const path = looksLikeSemverRange(version)
      ? `/v1/workflows/${encodeURIComponent(name)}/resolve?range=${encodeURIComponent(version)}`
      : `/v1/workflows/${encodeURIComponent(name)}/versions/${encodeURIComponent(version)}`;
    return this.request("GET", path);
  }

  async reportExternalOutcome(
    runId: string,
    bindingId: string,
    outcome: ExternalOutcomeReport,
    options: { webhookSecret?: string; idempotencyKey?: string } = {},
  ): Promise<Record<string, unknown>> {
    const body = JSON.stringify(outcome);
    const headers: Record<string, string> = {};
    if (options.webhookSecret) {
      headers["X-ArcFlow-Signature"] = await hmacSha256(options.webhookSecret, body);
    }
    if (options.idempotencyKey) {
      headers["Idempotency-Key"] = options.idempotencyKey;
    }
    return this.request("POST", `/v1/runs/${runId}/external/${bindingId}`, JSON.parse(body), headers);
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    extraHeaders?: Record<string, string>,
  ): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers: this.headers(extraHeaders),
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!res.ok) {
      const detail = await res.text();
      throw new StaticExecutionError(`[ArcFlow] ${method} ${path} failed (${res.status}): ${detail}`);
    }
    return (await res.json()) as T;
  }
}

function parseSiteIdFromBase(baseUrl: string): string | undefined {
  const match = baseUrl.match(/\/v1\/sites\/([^/]+)$/);
  return match?.[1];
}

function looksLikeSemverRange(version: string): boolean {
  return /^[<>=]/.test(version) || /[\^~*]/.test(version);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function hmacSha256(secret: string, body: string): Promise<string> {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(body));
  const hex = [...new Uint8Array(sig)].map((b) => b.toString(16).padStart(2, "0")).join("");
  return `sha256=${hex}`;
}
