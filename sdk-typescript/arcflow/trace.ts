export interface TokenUsage {
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
}

export interface StepTrace {
  stepIndex: number;
  agentName: string;
  agentRole: string;
  status: string;
  startedAt: Date;
  completedAt: Date | null;
  durationSeconds: number;
  tokensConsumed: TokenUsage;
}

export interface TraceResult {
  runId: string;
  workflowName: string;
  status: string;
  startedAt: Date;
  completedAt: Date | null;
  totalDurationSeconds: number;
  totalTokensConsumed: number;
  steps: StepTrace[];
  warnings: string[];
}

function asDict(value: unknown): Record<string, unknown> {
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : {};
}

function asInt(value: unknown, fallback = 0): number {
  return typeof value === "number" ? value : fallback;
}

function parseTs(value: unknown): Date {
  return new Date(String(value).replace("Z", "+00:00"));
}

function msToSeconds(value: unknown): number {
  return typeof value === "number" ? value / 1000 : 0;
}

function parseStep(raw: Record<string, unknown>): StepTrace {
  const tokens = asDict(raw.tokens);
  let status = String(raw.status ?? "Completed");
  if (status.includes("InProgress")) {
    status = "completed";
  } else {
    status = status.toLowerCase();
  }
  return {
    stepIndex: asInt(raw.step_index),
    agentName: String(raw.agent_name ?? ""),
    agentRole: String(raw.agent_role ?? ""),
    status,
    startedAt: parseTs(raw.started_at),
    completedAt: raw.completed_at ? parseTs(raw.completed_at) : null,
    durationSeconds: msToSeconds(raw.duration_ms),
    tokensConsumed: {
      promptTokens: asInt(tokens.prompt_tokens),
      completionTokens: asInt(tokens.completion_tokens),
      totalTokens: asInt(tokens.total_tokens),
    },
  };
}

export function traceFromJson(raw: string): TraceResult {
  const data = JSON.parse(raw) as Record<string, unknown>;
  const stepsRaw = Array.isArray(data.steps) ? data.steps : [];
  const steps = stepsRaw
    .filter((item): item is Record<string, unknown> => typeof item === "object" && item !== null)
    .map(parseStep);
  const totalTokens = asDict(data.total_tokens);
  const dropped = asInt(data.events_dropped);
  const warnings: string[] = [];
  if (dropped > 0) {
    warnings.push(`${dropped} trace events dropped (store capacity)`);
  }
  let status = String(data.status ?? "partial");
  if (status.startsWith("{")) {
    status = "partial";
  } else {
    status = status.toLowerCase();
  }
  return {
    runId: String(data.run_id),
    workflowName: String(data.workflow_name ?? "unknown"),
    status,
    startedAt: parseTs(data.started_at),
    completedAt: data.completed_at ? parseTs(data.completed_at) : null,
    totalDurationSeconds: msToSeconds(data.duration_ms),
    totalTokensConsumed: asInt(totalTokens.total_tokens),
    steps,
    warnings,
  };
}
