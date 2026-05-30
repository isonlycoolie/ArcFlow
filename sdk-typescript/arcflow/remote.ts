/** HTTP client for remote ArcFlow server execution. */

import { WorkflowConfigurationError, WorkflowExecutionError } from "./exceptions.js";
import { toWorkflowResult, type WorkflowResult } from "./result.js";
import type { Workflow } from "./workflow.js";

const API_KEY_HEADER = "X-ArcFlow-Api-Key";

interface CreateRunResponse {
  run_id: string;
  status: string;
}

interface RunStatusResponse {
  run_id: string;
  status: string;
  result?: { output: string; step_count: number };
  error?: { message: string; step_id?: string };
}

export async function publishRemoteWorkflow(
  workflow: Workflow,
  version: string,
  publishedBy?: string,
): Promise<Record<string, unknown>> {
  const baseUrl = workflow.runtimeUrl;
  if (!baseUrl) {
    throw new WorkflowConfigurationError(
      "[ArcFlow] Remote runtime URL is not configured.",
    );
  }
  const payload = workflow.buildPublishPayload(publishedBy);
  return requestJson(
    "PUT",
    `${baseUrl}/v1/workflows/${workflow.name}/versions/${version}`,
    payload,
  );
}

export async function resolveRemoteWorkflow(
  name: string,
  version: string,
  runtime: string,
): Promise<Record<string, unknown>> {
  const baseUrl = runtime.trim().replace(/\/$/, "");
  if (!baseUrl) {
    throw new WorkflowConfigurationError(
      "[ArcFlow] resolve() requires a non-empty runtime URL.",
    );
  }
  if (looksLikeSemverRange(version)) {
    const query = encodeURIComponent(version);
    return requestJson(
      "GET",
      `${baseUrl}/v1/workflows/${name}/resolve?range=${query}`,
    );
  }
  return requestJson(
    "GET",
    `${baseUrl}/v1/workflows/${name}/versions/${version}`,
  );
}

function looksLikeSemverRange(version: string): boolean {
  return /^[<>=]/.test(version) || /[\^~*]/.test(version);
}

export async function runRemoteWorkflow(
  workflow: Workflow,
  input: string,
  execConfigJson?: string,
): Promise<WorkflowResult> {
  const baseUrl = workflow.runtimeUrl;
  if (!baseUrl) {
    throw new WorkflowConfigurationError(
      "[ArcFlow] Remote runtime URL is not configured.",
    );
  }
  const payload = workflow.buildRunPayload(input, execConfigJson);
  const created = await requestJson<CreateRunResponse>(
    "POST",
    `${baseUrl}/v1/runs`,
    payload,
  );
  const detail = await requestJson<RunStatusResponse>(
    "GET",
    `${baseUrl}/v1/runs/${created.run_id}`,
  );
  if (detail.status.toLowerCase() === "failed") {
    throw new WorkflowExecutionError(
      detail.error?.message ?? "remote run failed",
      detail.run_id,
      detail.error?.step_id,
    );
  }
  return toWorkflowResult({
    output: detail.result?.output ?? "",
    runId: detail.run_id,
    stepCount: detail.result?.step_count ?? 0,
    traceEventsJson: "[]",
  });
}

async function requestJson<T>(
  method: string,
  url: string,
  body?: unknown,
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const apiKey = process.env.ARCFLOW_SERVER_API_KEY;
  if (apiKey) {
    headers[API_KEY_HEADER] = apiKey;
  }
  const response = await fetch(url, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    const detail = await response.text();
    throw new WorkflowConfigurationError(
      `[ArcFlow] Remote request failed (${response.status}): ${detail}`,
    );
  }
  return (await response.json()) as T;
}
